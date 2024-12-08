use std::collections::HashMap;

use base::{
    grids::Grid,
    ldtk::{grid_from_layer, EntityDef, EntityOnMap, GroundType, TerrainType, LDTK},
    FPos, Rect,
};
use nanoserde::DeJson;

use crate::{
    game::GameState,
    sprite::{self, Sprite},
    GRIDSIZE,
};

/// not dropped across reloads
pub struct PersistentState {
    // ######### BEGIN: data loaded from assets #########
    pub sprites: HashMap<String, Sprite>,
    pub ground: Grid<GroundType>,
    pub terrain: Grid<TerrainType>,
    pub ground_tiles: Vec<Tile>,
    pub terrain_tiles: Vec<Tile>,
    // ######### END  : data loaded from assets #########
    /// need to smuggle this into coroutines and have not found a better way
    pub delta: f32,

    /// the actual game state we would save
    pub g: GameState,
}

pub struct Tile {
    pub source_rect: Rect,
    pub pos: FPos,
}

impl PersistentState {
    pub fn new() -> Self {
        let input = std::fs::read_to_string("../assets/comfy_wars.ldtk").unwrap();
        let ldtk: LDTK = DeJson::deserialize_json(&input).unwrap();

        let ground: Grid<GroundType>;
        let terrain: Grid<TerrainType>;
        let ground_tiles: Vec<Tile>;
        let terrain_tiles: Vec<Tile>;
        {
            let layer = ldtk
                .levels
                .iter()
                .flat_map(|level| level.layers.iter())
                .filter(|layer| layer.id == "groundgrid")
                .next()
                .unwrap();
            ground = grid_from_layer(layer, |i| match i {
                1 => GroundType::Ground,
                2 => GroundType::Water,
                _ => panic!("unsupported ground type {}", i),
            });
            ground_tiles = layer
                .auto_tiles
                .iter()
                .map(|tile| {
                    let source_rect = Rect {
                        x: tile.src[0] as _,
                        y: tile.src[1] as _,
                        w: GRIDSIZE as _,
                        h: GRIDSIZE as _,
                    };
                    let pos = FPos { x: tile.px[0], y: tile.px[1] };
                    Tile { source_rect, pos }
                })
                .collect();
        }

        {
            let layer = ldtk
                .levels
                .iter()
                .flat_map(|level| level.layers.iter())
                .filter(|layer| layer.id == "infrastructuregrid")
                .next()
                .unwrap();
            terrain = grid_from_layer(layer, |i| match i {
                0 => TerrainType::None,
                1 | 2 | 3 | 4 => TerrainType::Street,
                5 => TerrainType::Forest,
                _ => panic!("unsupported terrain type {}", i),
            });

            terrain_tiles = layer
                .auto_tiles
                .iter()
                .map(|tile| {
                    let source_rect = Rect {
                        x: tile.src[0] as _,
                        y: tile.src[1] as _,
                        w: GRIDSIZE as _,
                        h: GRIDSIZE as _,
                    };
                    let pos = FPos { x: tile.px[0], y: tile.px[1] };
                    Tile { source_rect, pos }
                })
                .collect();
        }

        let g = GameState::new();

        Self {
            sprites: sprite::load_sprites("../assets/sprites.json"),
            ground,
            terrain,
            ground_tiles,
            terrain_tiles,
            g,
            delta: 0.0,
        }
    }
}
