use std::collections::HashMap;

use base::{
    ldtk::{EntityDef, EntityOnMap, Team, UnitType},
    Color, ContextTrait, FPos, Key, Pos, Rect,
};
use nanoserde::DeJson;

use crate::{fleeting::FleetingState, genarena::GenArena, persistent::PersistentState, util::grid_world_pos, GRIDSIZE};

const HP_MAX: i32 = 10;

pub struct GameState {
    pub actors: GenArena<Actor>,
}

pub struct Actor {
    pub pos: Pos,
    pub draw_pos: FPos,
    pub sprite: String,
    pub team: Team,
    pub unit_type: UnitType,
    pub hp: i32,
    pub has_moved: bool,
}

impl GameState {
    pub fn new() -> Self {
        // load actors
        let mut actors = GenArena::new();
        {
            let input = std::fs::read_to_string("../assets/entities_def.json").unwrap();
            let entity_defs: HashMap<String, EntityDef> = DeJson::deserialize_json(&input).unwrap();
            // load entities on map
            let input = std::fs::read_to_string("../assets/entities_map.json").unwrap();
            let map_entities: Vec<EntityOnMap> = DeJson::deserialize_json(&input).unwrap();

            for me in map_entities {
                let name = &me.def;
                let def = &entity_defs[&me.def];
                let a = Actor {
                    pos: Pos {
                        x: me.pos[0],
                        y: me.pos[1],
                    },
                    draw_pos: FPos {
                        x: me.pos[0] as f32 * GRIDSIZE,
                        y: me.pos[1] as f32 * GRIDSIZE,
                    },
                    sprite: name.clone(),
                    team: def.team,
                    unit_type: def.unit_type,
                    hp: HP_MAX,
                    has_moved: false,
                };
                actors.push(a);
            }
        }
        GameState { actors }
    }
}

pub fn update_inner(c: &mut dyn ContextTrait, s: &mut PersistentState, f: &mut FleetingState) {
    f.queue.run_until_stall(s);

    for tile in &s.ground_tiles {
        c.draw_texture("tiles", tile.source_rect, tile.pos.x, tile.pos.y, 0);
    }
    for tile in &s.terrain_tiles {
        c.draw_texture("tiles", tile.source_rect, tile.pos.x, tile.pos.y, 1);
    }

    if c.is_pressed(Key::MouseLeft) {
        s.sprites["arrow_s"].draw(c, 80., 50., 1);
    }
    let pos = grid_world_pos(c.mouse_world());
    s.sprites["cursor"].draw(c, pos.x, pos.y, 1);

    for actor in s.g.actors.iter() {
	let sprite = &s.sprites[&actor.sprite];
	sprite.draw(c, actor.draw_pos.x, actor.draw_pos.y, 10);
    }
}
