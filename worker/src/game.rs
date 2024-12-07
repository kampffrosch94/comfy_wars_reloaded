use base::{Color, ContextTrait, Key, Rect};

use crate::{fleeting::FleetingState, persistent::PersistentState, GRIDSIZE};

// TODO
struct GameState {}

pub fn update_inner(c: &mut dyn ContextTrait, s: &mut PersistentState, f: &mut FleetingState) {
    f.queue.run_until_stall(s);

    s.sprites["red_infantry"].draw(c, GRIDSIZE*1., GRIDSIZE * 1., 1);

    for tile in &s.ground_tiles {
        c.draw_texture("tiles", tile.source_rect, tile.pos.x, tile.pos.y, 0);
    }
    for tile in &s.terrain_tiles {
        c.draw_texture("tiles", tile.source_rect, tile.pos.x, tile.pos.y, 1);
    }

    if c.is_pressed(Key::MouseLeft) {
        s.sprites["arrow_s"].draw(c, 80., 50., 1);
    }
}
