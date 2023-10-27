//! Some common tile map providers.

use crate::mercator::TileId;

#[derive(Clone, Copy, PartialEq)]
pub enum MapType {
    Standart,
    Satellite,
    Hybrid,
    Roads,
    Terrain,
}

impl MapType {
    pub fn to_string(&self) -> String {
        let s = match self {
            MapType::Standart => "Standart",
            MapType::Satellite => "Satellite",
            MapType::Hybrid => "Hybrid",
            MapType::Roads => "Roads",
            MapType::Terrain => "Terrain",
        };

        s.to_string()
    }
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

/// <https://www.openstreetmap.org/about>
pub struct OpenStreetMap;

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

/// Orthophotomap layer from Poland's Geoportal.
/// <https://www.geoportal.gov.pl/uslugi/usluga-przegladania-wms>
pub struct Geoportal;

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

pub struct Google {
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
