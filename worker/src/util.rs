#![allow(unused)]
use base::{FPos, Pos};

use crate::GRIDSIZE;

/// translates from world coordinates to game grid
pub fn world_to_game(p: FPos) -> Pos {
    let x = (p.x / GRIDSIZE) as _;
    let y = (p.y / GRIDSIZE) as _;
    Pos { x, y }
}

/// rounds pos to align with grid
pub fn grid_world_pos(p: FPos) -> FPos {
    let x = (p.x / GRIDSIZE).floor() * GRIDSIZE;
    let y = (p.y / GRIDSIZE).floor() * GRIDSIZE;
    FPos { x, y }
}
