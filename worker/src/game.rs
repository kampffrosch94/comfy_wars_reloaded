use std::collections::{HashMap, HashSet};

use base::{
    grids::Grid,
    ldtk::{EntityDef, EntityOnMap, GroundType, Team, TerrainType, UnitType},
    Button, Color, ContextTrait, FPos, Pos, Rect,
};
use nanoserde::DeJson;

use crate::{
    dijkstra::{dijkstra, dijkstra_path},
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
    Selected(Key<Actor>),
    Moving(Key<Actor>),
    Confirm(Key<Actor>),
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
    s.delta = c.delta();
    f.co.run_until_stall(s);

    draw_nine_patch(c, "ui_bg", 16., Rect {x: 0., y: 500., w: 100., h: 70.});
    draw_nine_patch(c, "ui_bg", 16., Rect {x: 0., y: 600., w: 32., h: 100.});
    draw_nine_patch(c, "ui_bg", 16., Rect {x: 100., y: 600., w: 180., h: 180.});

    //c.draw_rect(rect, c, z_level);
    // TODO add Z to draw text
    // c.draw_text("Hello World asfsj", 0., 500., 50);

    for tile in &s.ground_tiles {
        c.draw_texture_part("tiles", tile.source_rect, tile.pos.x, tile.pos.y, 0);
    }
    for tile in &s.terrain_tiles {
        c.draw_texture_part("tiles", tile.source_rect, tile.pos.x, tile.pos.y, 1);
    }

    for actor in s.g.actors.iter() {
        let sprite = &s.sprites[&actor.sprite];
        sprite.draw(c, actor.draw_pos.x, actor.draw_pos.y, 10);
    }

    match s.g.selection {
        Selection::None => {
            let pos = grid_world_pos(c.mouse_world());
            s.sprites["cursor"].draw(c, pos.x, pos.y, 10);
            // select actor
            if c.is_pressed(Button::MouseLeft) {
                let pos = world_to_game(c.mouse_world());
                if let Some((key, _)) =
                    s.g.actors
                        .iter_keys()
                        .filter(|(_key, a)| a.pos == pos && a.team == Team::Blue)
                        .next()
                {
                    s.g.selection = Selection::Selected(key);
                }
            }
        }
        Selection::Selected(key) => {
            let a = &s.g.actors[key];
            s.sprites["cursor"].draw(c, a.draw_pos.x, a.draw_pos.y, 10);

            // draw moveable area
            let start_pos = a.pos;
            let mut move_range = Grid::new(s.ground.width, s.ground.height, 0);
            move_range[start_pos] = 9;
            dijkstra(&mut move_range, &[start_pos], movement_cost(s, PLAYER_TEAM));
            draw_move_range(c, s, &move_range);

            // find goal
            let mut grid = Grid::new(s.ground.width, s.ground.height, 0);
            let goal = world_to_game(c.mouse_world());
            *grid.get_clamped_mut(goal.x, goal.y) = 99; // TODO increase this when done developing
            dijkstra(&mut grid, &[goal], movement_cost(s, PLAYER_TEAM));
            move_range.clamp_values(0, 1);
            grid.mul_inplace(&move_range);

            // allow passing through allies, but don't stop on them
            let mut seeds = Vec::new();
            for actor in s.g.actors.iter() {
                grid[actor.pos] = -99;
                seeds.push(actor.pos);
            }
            let highest_reachable_pos = grid
                .iter_coords()
                .max_by_key(|(_pos, val)| *val)
                .map(|(pos, _)| pos)
                .unwrap();
            seeds.push(highest_reachable_pos);
            dijkstra(&mut grid, &seeds, movement_cost(s, PLAYER_TEAM));
            grid.mul_inplace(&move_range);

            // disallow moving through enemies
            for actor in s.g.actors.iter().filter(|a| a.team != PLAYER_TEAM) {
                grid[actor.pos] = -99;
            }

            // finally actually calculate and draw the path
            let path = dijkstra_path(&grid, start_pos);
            draw_move_range(c, s, &grid);
            draw_move_path(c, s, &path);
            if c.is_pressed(Button::MouseLeft) && path.len() > 0 {
                s.g.selection = Selection::Moving(key);
                f.co.queue(move |mut s| async move {
                    for pos in path.iter() {
                        let target = game_to_world(*pos);
                        let mut lerpiness = 0.;
                        while lerpiness < 1. {
                            {
                                let s = &mut s.get();
                                lerpiness += s.delta * 25.;
                                let drawpos = &mut s.g.actors[key].draw_pos;
                                *drawpos = drawpos.lerp(target.into(), lerpiness);
                            }
                            cosync::sleep_ticks(1).await;
                        }
                    }
                    let last = *path.last().unwrap();
                    let target = game_to_world(last);
                    let s = &mut s.get();
                    s.g.actors[key].draw_pos = target.into();
                    s.g.actors[key].pos = last;
                    s.g.selection = Selection::Confirm(key);
                });
            }
        }
        Selection::Moving(key) => {
            let _a = &s.g.actors[key];
            // TODO
        }
        Selection::Confirm(key) => {
            let a = &s.g.actors[key];
            // TODO
            s.g.selection = Selection::None;
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

fn draw_move_path(c: &mut dyn ContextTrait, s: &PersistentState, path: &[Pos]) {
    const DOWN: (i32, i32) = (0, 1);
    const UP: (i32, i32) = (0, -1);
    const RIGHT: (i32, i32) = (1, 0);
    const LEFT: (i32, i32) = (-1, 0);

    let mut iter = path.iter();
    let prev = iter.next().cloned();
    let mut prev_direction: Option<(i32, i32)> = None;
    if let Some(mut prev) = prev {
        for pos in iter {
            let direction = (*pos - prev).into();
            if let Some(prev_direction) = prev_direction {
                let sprite = match (prev_direction, direction) {
                    (LEFT, LEFT) | (RIGHT, RIGHT) => "arrow_we",
                    (UP, UP) | (DOWN, DOWN) => "arrow_ns",
                    (DOWN, RIGHT) | (LEFT, UP) => "arrow_ne",
                    (UP, RIGHT) | (LEFT, DOWN) => "arrow_se",
                    (DOWN, LEFT) | (RIGHT, UP) => "arrow_wn",
                    (UP, LEFT) | (RIGHT, DOWN) => "arrow_ws",
                    _ => panic!("should be impossible"),
                };
                let sprite = &s.sprites[sprite];
                let draw_pos = game_to_world(prev);
                sprite.draw(c, draw_pos.x, draw_pos.y, 10);
            }
            prev = *pos;
            prev_direction = Some(direction);
        }
    }

    // draw ending arrow
    let len = path.len();
    if len >= 2 {
        let prev = path[path.len() - 2];
        let pos = path[path.len() - 1];
        let direction: (i32, i32) = (pos - prev).into();
        let sprite = match direction {
            LEFT => "arrow_w",
            RIGHT => "arrow_e",
            DOWN => "arrow_s",
            UP => "arrow_n",
            _ => panic!("should be impossible"),
        };
        let sprite = &s.sprites[sprite];
        let draw_pos = game_to_world(pos);
        sprite.draw(c, draw_pos.x, draw_pos.y, 10);
    }
}

#[rustfmt::skip]
fn draw_nine_patch(c: &mut dyn ContextTrait, texture: &str, corner: f32, trect: Rect) {
    let z = 100;
    let source_rect = Rect {x: 0., y: 0., w: 192., h: 64.,};


    // corners
    let tl = source_rect.take_left(corner).take_top(corner);
    let tr = source_rect.take_right(corner).take_top(corner);
    let bl = source_rect.take_left(corner).take_bot(corner);
    let br = source_rect.take_right(corner).take_bot(corner);

    c.draw_texture_part(texture, tl, trect.x, trect.y, z);
    c.draw_texture_part(texture, tr, trect.take_right(corner).x, trect.y, z);
    c.draw_texture_part(texture, bl, trect.x, trect.y + trect.h - corner, z);
    c.draw_texture_part(texture, br, trect.take_right(corner).x, trect.take_bot(corner).y, z);

    // middle sides
    let amount = corner;
    // top middle
    let source = source_rect.skip_left(corner).take_top(corner).skip_right(corner);
    let target = trect.take_top(amount).skip_left(amount).skip_right(amount);
    c.draw_texture_part_scaled(texture, source, target, z);
    // bot middle
    let source = source_rect.skip_left(corner).take_bot(corner).skip_right(corner);
    let target = trect.take_bot(amount).skip_left(amount).skip_right(amount);
    c.draw_texture_part_scaled(texture, source, target, z);
    // left middle
    let source = source_rect.skip_top(corner).take_left(corner).skip_bot(corner);
    let target = trect.skip_top(amount).skip_bot(amount).take_left(amount);
    c.draw_texture_part_scaled(texture, source, target, z);
    // right middle
    let source = source_rect.skip_top(corner).take_right(corner).skip_bot(corner);
    let target = trect.skip_top(amount).skip_bot(amount).take_right(amount);
    c.draw_texture_part_scaled(texture, source, target, z);
    // center 
    let source = source_rect.skip_top(corner).skip_right(corner)
	.skip_bot(corner).skip_left(corner);
    let target = trect.skip_top(corner).skip_right(corner)
	.skip_bot(corner).skip_left(corner);
    c.draw_texture_part_scaled(texture, source, target, z);



    //c.draw_text(&format!("{source:?}"), 0., 600., 50);
}
