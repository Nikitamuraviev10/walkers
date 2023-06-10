use std::{
    collections::{hash_map::Entry, HashMap},
    ops::Deref,
};

use egui::{Mesh, Painter, Pos2, Response, Sense, Ui, Widget};

use crate::{
    mercator::{screen_to_position, PositionExt, TileId},
    Position, Tiles,
};

/// Slippy map widget.
pub struct Map<'a, 'b> {
    tiles: &'b mut Tiles,
    memory: &'a mut MapMemory,
    my_position: Position,
}

impl<'a, 'b> Map<'a, 'b> {
    pub fn new(tiles: &'b mut Tiles, memory: &'a mut MapMemory, my_position: Position) -> Self {
        Self {
            tiles,
            memory,
            my_position,
        }
    }
}

impl Widget for Map<'_, '_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());

        self.memory
            .center_mode
            .screen_drag(&response, self.my_position, *self.memory.zoom);

        let map_center = self.memory.center_mode.position(self.my_position);
        let painter = ui.painter().with_clip_rect(rect);

        if self.memory.osm {
            let mut meshes = Default::default();
            draw_tiles(
                &painter,
                map_center.tile_id(*self.memory.zoom),
                map_center.project_with_zoom(*self.memory.zoom).into(),
                self.tiles,
                ui,
                &mut meshes,
            );

            for (_, shape) in meshes {
                painter.add(shape);
            }
        }

        response
    }
}

#[derive(Clone, PartialEq)]
pub enum MapCenterMode {
    MyPosition,
    Exact(Position),
}

impl MapCenterMode {
    fn screen_drag(&mut self, response: &Response, my_position: Position, zoom: u8) {
        if response.dragged_by(egui::PointerButton::Primary) {
            *self = match *self {
                MapCenterMode::MyPosition => MapCenterMode::Exact(my_position),
                MapCenterMode::Exact(position) => MapCenterMode::Exact({
                    let position_delta = screen_to_position(response.drag_delta(), zoom);
                    Position::new(
                        position.x() - position_delta.x(),
                        position.y() - position_delta.y(),
                    )
                }),
            };
        }
    }

    pub fn position(&self, my_position: Position) -> Position {
        match self {
            MapCenterMode::MyPosition => my_position,
            MapCenterMode::Exact(position) => *position,
        }
    }
}

#[derive(Clone)]
pub struct MapMemory {
    pub center_mode: MapCenterMode,
    pub osm: bool,
    pub zoom: Zoom,
}

impl Default for MapMemory {
    fn default() -> Self {
        Self {
            center_mode: MapCenterMode::MyPosition,
            osm: false,
            zoom: Default::default(),
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("invalid zoom level")]
pub struct InvalidZoom;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Zoom(u8);

impl TryFrom<u8> for Zoom {
    type Error = InvalidZoom;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        // OSM wiki has level 20 listed https://wiki.openstreetmap.org/wiki/Zoom_levels,
        // but when requested, server responds with 400: Bad Request.
        if value > 19 {
            Err(InvalidZoom)
        } else {
            Ok(Self(value))
        }
    }
}

impl Default for Zoom {
    fn default() -> Self {
        Self(16)
    }
}

impl Deref for Zoom {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Zoom {
    pub fn try_zoom_in(&mut self) -> Result<(), InvalidZoom> {
        *self = Self::try_from(self.0 + 1)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructing_zoom() {
        assert_eq!(16, *Zoom::default());
        assert_eq!(19, *Zoom::try_from(19).unwrap());
        assert_eq!(Err(InvalidZoom), Zoom::try_from(20));
    }

    #[test]
    fn test_zooming_in_and_out() {
        let mut zoom = Zoom::try_from(18).unwrap();
        assert!(zoom.try_zoom_in().is_ok());
        assert_eq!(Err(InvalidZoom), zoom.try_zoom_in());
    }
}

fn draw_tiles(
    painter: &Painter,
    tile_id: TileId,
    map_center_projected_position: Pos2,
    tiles: &mut Tiles,
    ui: &mut Ui,
    meshes: &mut HashMap<TileId, Mesh>,
) {
    let tile_projected = tile_id.position_on_world_bitmap();
    let tile_screen_position = painter.clip_rect().center().to_vec2() + tile_projected.to_vec2()
        - map_center_projected_position.to_vec2();

    let image = if let Some(image) = tiles.at(tile_id) {
        image
    } else {
        return;
    };

    if painter
        .clip_rect()
        .intersects(image.rect(tile_screen_position))
    {
        if let Entry::Vacant(vacant) = meshes.entry(tile_id) {
            vacant.insert(image.mesh(tile_screen_position, ui.ctx()));

            for coordinates in [
                tile_id.north(),
                tile_id.east(),
                tile_id.south(),
                tile_id.west(),
            ] {
                draw_tiles(
                    painter,
                    coordinates,
                    map_center_projected_position,
                    tiles,
                    ui,
                    meshes,
                );
            }
        }
    }
}
