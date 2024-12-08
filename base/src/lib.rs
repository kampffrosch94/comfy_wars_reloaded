use std::{ffi::c_void, ops::Sub};
pub mod grids;
pub mod ldtk;
pub mod rect;

pub trait ContextTrait {
    /// time since program start
    fn time(&self) -> f64;

    /// frame delta time
    fn delta(&self) -> f32;

    /// frames per second
    fn fps(&self) -> f32;

    fn draw_rect(&mut self, rect: Rect, c: Color, z_level: i32);

    fn draw_text(&mut self, text: &str, size: f32, x: f32, y: f32, z_level: i32);

    fn draw_texture(&mut self, name: &str, x: f32, y: f32, z_level: i32);

    fn draw_texture_part(&mut self, name: &str, src: Rect, x: f32, y: f32, z_level: i32);

    fn draw_texture_part_scaled(&mut self, name: &str, src: Rect, target: Rect, z_level: i32);

    fn load_texture(&mut self, name: &str, path: &str);

    fn texture_dimensions(&mut self, name: &str) -> Rect;

    fn is_pressed(&self, button: Button) -> bool;

    fn mouse_screen(&self) -> FPos;

    fn mouse_world(&self) -> FPos;
}

pub enum Button {
    MouseLeft,
    MouseMiddle,
    MouseRight,
}

#[derive(Debug, Clone, Copy)]
pub struct FPos {
    pub x: f32,
    pub y: f32,
}

impl FPos {
    pub fn lerp(self, rhs: Self, s: f32) -> Self {
        let x = self.x + ((rhs.x - self.x) * s);
        let y = self.y + ((rhs.y - self.y) * s);
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Sub<Pos> for Pos {
    type Output = (i32, i32);

    fn sub(self, rhs: Pos) -> Self::Output {
        (self.x - rhs.x, self.y - rhs.y)
    }
}

/// x and y are in the top left
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Wrapper for state that is persisted between reloads
#[repr(C)]
pub struct PersistWrapper {
    pub ptr: *mut c_void,
    pub size: usize,
    pub align: usize,
}

impl PersistWrapper {
    pub fn ref_mut<T>(&mut self) -> &mut T {
        // TODO add checks for size and alignment matching
        let ptr = self.ptr as *mut T;
        unsafe { &mut *ptr }
    }
}
