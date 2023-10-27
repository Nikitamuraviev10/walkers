//! Some common tile map providers.
use crate::mercator::TileId;
use enum2str::EnumStr;
pub type Provider = Box<dyn TileSource + Send>;

#[derive(EnumStr, Clone, Copy, PartialEq)]
pub enum MapType {
    Standart,
    Satellite,
    Hybrid,
    Roads,
    Terrain,
}

#[derive(EnumStr, Clone, Copy, PartialEq)]
pub enum Providers {
    /// <https://www.openstreetmap.org/about>
    OpenStreetMap,

    /// Orthophotomap layer from Poland's Geoportal.
    /// <https://www.geoportal.gov.pl/uslugi/usluga-przegladania-wms>
    Geoportal,

    /// <https://www.google.com/maps/about>
    Google,
}

#[derive(Clone, Copy)]
pub struct Attribution {
    pub text: &'static str,
    pub url: &'static str,
}

pub trait TileSource {
    fn tile_url(&self, tile_id: TileId) -> String;
    fn attribution(&self) -> Attribution;
}

struct OpenStreetMap;

impl TileSource for OpenStreetMap {
    fn tile_url(&self, tile_id: TileId) -> String {
        format!(
            "https://tile.openstreetmap.org/{}/{}/{}.png",
            tile_id.zoom, tile_id.x, tile_id.y
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "OpenStreetMap contributors",
            url: "https://www.openstreetmap.org/copyright",
        }
    }
}

struct Geoportal;

impl TileSource for Geoportal {
    fn tile_url(&self, tile_id: TileId) -> String {
        format!(
            "https://mapy.geoportal.gov.pl/wss/service/PZGIK/ORTO/WMTS/StandardResolution?\
            &SERVICE=WMTS\
            &REQUEST=GetTile\
            &VERSION=1.0.0\
            &LAYER=ORTOFOTOMAPA\
            &TILEMATRIXSET=EPSG:3857\
            &TILEMATRIX=EPSG:3857:{}\
            &TILEROW={}\
            &TILECOL={}",
            tile_id.zoom, tile_id.y, tile_id.x
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "Główny Urząd Geodezji i Kartografii",
            url: "https://www.geoportal.gov.pl/",
        }
    }
}

struct Google {
    t: char,
}

impl Google {
    pub fn new(map_type: MapType) -> Self {
        let t = match map_type {
            MapType::Standart => 'm',
            MapType::Satellite => 's',
            MapType::Hybrid => 'y',
            MapType::Roads => 'h',
            MapType::Terrain => 'p',
        };

        Self { t }
    }
}

impl TileSource for Google {
    fn tile_url(&self, tile_id: TileId) -> String {
        format!(
            "http://mt1.google.com/vt/lyrs={}&x={}&y={}&z={}",
            self.t, tile_id.x, tile_id.y, tile_id.zoom
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "Google Maps",
            url: "https://www.google.com/maps/",
        }
    }
}

/// Constructor function for map providers
pub fn new(provider: Providers, map_type: MapType) -> Provider {
    match provider {
        Providers::OpenStreetMap => Box::new(OpenStreetMap),
        Providers::Geoportal => Box::new(Geoportal),
        Providers::Google => Box::new(Google::new(map_type)),
    }
}
