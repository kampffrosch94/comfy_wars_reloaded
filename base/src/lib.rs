use std::ffi::c_void;

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

pub trait ContextTrait {
    fn draw_rect(&self, r: Rect, c: Color);

    /// time since program start
    fn time(&self) -> f64;

    fn draw_text(&self, text: &str, x: f32, y: f32);
}

/// Wrapper for state that is persisted between reloads
#[repr(C)]
pub struct PersistWraper {
    pub ptr: *mut c_void,
    pub size: usize,
    pub align: usize,
}

impl PersistWraper {
    pub fn ref_mut<T>(&mut self) -> &mut T {
        // TODO add checks for size and alignment matching
        let ptr = self.ptr as *mut T;
        unsafe { &mut *ptr }
    }
}
