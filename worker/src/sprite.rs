use std::collections::HashMap;

use base::{ContextTrait, Rect};
use nanoserde::DeJson;

#[derive(Clone)]
pub struct Sprite {
    src: Rect,
}

impl Sprite {
    pub fn draw(&self, c: &mut dyn ContextTrait, x: f32, y: f32, z_level: i32) {
        c.draw_texture("tiles", self.src, x, y, z_level)
    }
}

#[derive(DeJson, Debug)]
pub struct SpriteData {
    pub x: i32,
    pub y: i32,
}

const GRID_SIZE: f32 = 16.;

pub fn load_sprites(path: &str) -> HashMap<String, Sprite> {
    let input = std::fs::read_to_string(path).unwrap();
    let loaded: HashMap<String, SpriteData> = DeJson::deserialize_json(&input).unwrap();
    loaded
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                Sprite {
                    src: Rect {
                        x: v.x as _,
                        y: v.y as _,
                        w: GRID_SIZE,
                        h: GRID_SIZE,
                    },
                },
            )
        })
        .collect()
}
