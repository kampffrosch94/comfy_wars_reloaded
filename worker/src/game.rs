use std::collections::{HashMap, HashSet};

use base::{
    grids::Grid,
    ldtk::{EntityDef, EntityOnMap, GroundType, Team, TerrainType, UnitType},
    Button, Color, ContextTrait, FPos, Pos, Rect,
};
use nanoserde::DeJson;

use crate::{
    dijkstra::dijkstra,
    fleeting::FleetingState,
    genarena::{GenArena, Key},
    persistent::PersistentState,
    util::{game_to_world, grid_world_pos, world_to_game},
    GRIDSIZE,
};

const HP_MAX: i32 = 10;
pub const ENEMY_TEAM: Team = Team::Red;
pub const PLAYER_TEAM: Team = Team::Blue;

pub struct GameState {
    pub actors: GenArena<Actor>,
    pub selection: Selection,
}

pub enum Selection {
    None,
    Unit(Key<Actor>),
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
        GameState {
            actors,
            selection: Selection::None,
        }
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

    for actor in s.g.actors.iter() {
        let sprite = &s.sprites[&actor.sprite];
        sprite.draw(c, actor.draw_pos.x, actor.draw_pos.y, 10);
    }

    // select actor
    if c.is_pressed(Button::MouseLeft) {
        let pos = world_to_game(c.mouse_world());
        if let Some((key, _)) =
            s.g.actors
                .iter_keys()
                .filter(|(_key, a)| a.pos == pos && a.team == Team::Blue)
                .next()
        {
            s.g.selection = Selection::Unit(key);
        }
    }

    // draw cursor
    match s.g.selection {
        Selection::None => {
            let pos = grid_world_pos(c.mouse_world());
            s.sprites["cursor"].draw(c, pos.x, pos.y, 10);
        }
        Selection::Unit(key) => {
            let a = &s.g.actors[key];
            s.sprites["cursor"].draw(c, a.draw_pos.x, a.draw_pos.y, 10);
            let pos = grid_world_pos(c.mouse_world());
            s.sprites["arrow_ne"].draw(c, pos.x, pos.y, 10);

            // draw moveable area
            let start_pos = a.pos;
            let mut move_range = Grid::new(s.ground.width, s.ground.height, 0);
            move_range[start_pos] = 9;
            dijkstra(&mut move_range, &[start_pos], movement_cost(s, PLAYER_TEAM));
	    draw_move_range(c, s, &move_range);
        }
    }
}

fn movement_cost<'a>(s: &'a PersistentState, team: Team) -> impl Fn(Pos) -> i32 + 'a {
    let blocked: HashSet<Pos> =
        s.g.actors
            .iter()
            .filter_map(|a| if a.team != team { Some(a.pos) } else { None })
            .collect();

    let cost_function = move |pos| -> i32 {
        if blocked.contains(&pos) {
            return 9999;
        }
        let ground = *s.ground.get_clamped_v(pos);
        let terrain = *s.terrain.get_clamped_v(pos);
        use GroundType as G;
        use TerrainType as T;
        match (ground, terrain) {
            (G::Water, _) => 9999,
            (G::Ground, T::None) => 2,
            (G::Ground, T::Street) => 1,
            (G::Ground, T::Forest) => 3,
        }
    };
    cost_function
}

fn draw_move_range(c: &mut dyn ContextTrait, s: &PersistentState, grid: &Grid<i32>) {
    for (x, y, v) in grid.iter() {
        if *v > 0 {
            let pos = Pos::new(x, y);
            let pos = game_to_world(pos);
            s.sprites["move_range"].draw(c, pos.x, pos.y, 2);
        }
    }
}
