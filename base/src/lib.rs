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

pub struct Context {
    pub draw: Box<fn(Rect, Color)>,
    pub time: Box<fn() -> f64>,
}

impl Context {
}
