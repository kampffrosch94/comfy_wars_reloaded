use std::cell::RefCell;

use base::*;

use macroquad::prelude::*;

use crate::{camera::CameraWrapper, util::texture_store::TextureStore};

#[derive(Default)]
pub struct Context {
    draw_buffer: RefCell<Vec<DrawCommand>>,
    pub camera: CameraWrapper,
    pub textures: TextureStore,
    pub loading: Vec<(String, String)>,
}

impl ContextTrait for Context {
    /// time since program start
    fn time(&self) -> f64 {
        get_time()
    }

    fn delta(&self) -> f32 {
        get_frame_time()
    }

    fn fps(&self) -> f32 {
        get_fps() as f32
    }

    fn draw_rect(&mut self, rect: base::Rect, c: base::Color, z_level: i32) {
        let color = macroquad::prelude::Color { r: c.r, g: c.g, b: c.b, a: c.a };

        let command = move || {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
        };
        self.draw_buffer
            .borrow_mut()
            .push(DrawCommand { z_level, command: Box::new(command) });
    }

    fn draw_text(&mut self, text: &str, size: f32, x: f32, y: f32, z_level: i32) {
        let text = text.to_string();
        let command = move || {
	    // TODO this tanks performance during zoom (maybe disable it then/cache it?)
            let (font_size, font_scale, font_aspect) = camera_font_scale(size);
            let text_params = TextParams {
                font_size,
                font_scale,
                font_scale_aspect: font_aspect,
                ..Default::default()
            };
            draw_text_ex(&text, x, y, text_params);
        };
        self.draw_buffer
            .borrow_mut()
            .push(DrawCommand { z_level, command: Box::new(command) });
    }

    fn draw_texture(&mut self, name: &str, x: f32, y: f32, z_level: i32) {
        // load if not in texture store
        // then add to draw buffer
        if let Some(texture) = self.textures.get(name) {
            let source = None;
            let params = DrawTextureParams { source, ..Default::default() };
            let command = move || {
                draw_texture_ex(&texture, x, y, WHITE, params);
            };
            self.draw_buffer
                .borrow_mut()
                .push(DrawCommand { z_level, command: Box::new(command) });
        } else {
            self.draw_text(&format!("ERROR('{name}')"), 20., x, y, 9999);
        }
    }

    fn draw_texture_part(
        &mut self,
        name: &str,
        src: base::Rect,
        x: f32,
        y: f32,
        z_level: i32,
    ) {
        // load if not in texture store
        // then add to draw buffer
        if let Some(texture) = self.textures.get(name) {
            let source =
                Some(macroquad::math::Rect { x: src.x, y: src.y, w: src.w, h: src.h });
            let params = DrawTextureParams { source, ..Default::default() };
            let command = move || {
                draw_texture_ex(&texture, x, y, WHITE, params);
            };
            self.draw_buffer
                .borrow_mut()
                .push(DrawCommand { z_level, command: Box::new(command) });
        } else {
            self.draw_text(&format!("ERROR('{name}')"), 20., x, y, 9999);
        }
    }

    fn draw_texture_part_scaled(
        &mut self,
        name: &str,
        src: base::Rect,
        target: base::Rect,
        z_level: i32,
    ) {
        // load if not in texture store
        // then add to draw buffer
        if let Some(texture) = self.textures.get(name) {
            let source =
                Some(macroquad::math::Rect { x: src.x, y: src.y, w: src.w, h: src.h });
            let dest_size = Some(vec2(target.w, target.h));
            let params = DrawTextureParams { source, dest_size, ..Default::default() };
            let command = move || {
                draw_texture_ex(&texture, target.x, target.y, WHITE, params);
            };
            self.draw_buffer
                .borrow_mut()
                .push(DrawCommand { z_level, command: Box::new(command) });
        } else {
            self.draw_text(&format!("ERROR('{name}')"), 20., target.x, target.y, 9999)
        }
    }

    fn is_pressed(&self, button: Button) -> bool {
        match button {
            Button::MouseLeft => is_mouse_button_pressed(MouseButton::Left),
            Button::MouseMiddle => is_mouse_button_pressed(MouseButton::Middle),
            Button::MouseRight => is_mouse_button_pressed(MouseButton::Right),
        }
    }

    fn mouse_screen(&self) -> FPos {
        let m = mouse_position();
        FPos { x: m.0, y: m.1 }
    }

    fn mouse_world(&self) -> FPos {
        let m = self.camera.mouse_world();
        FPos { x: m.x, y: m.y }
    }

    fn load_texture(&mut self, name: &str, path: &str) {
        self.loading.push((name.to_string(), path.to_string()));
    }

    fn texture_dimensions(&mut self, name: &str) -> base::Rect {
        self.textures
            .get(name)
            .map(|t| base::Rect { x: 0., y: 0., w: t.width(), h: t.width() })
            .unwrap_or(base::Rect { x: 0., y: 0., w: 0., h: 0. })
    }
}

impl Context {
    /// executes deferred drawing, should be called once per frame
    pub async fn process(&mut self) {
        for (name, path) in self.loading.drain(..) {
            if let Err(_err) = self.textures.load_texture(&path, name, false).await {
                println!("Error loading {}", &path);
            }
        }

        let buffer = &mut self.draw_buffer.borrow_mut();
        buffer.sort_by_key(|it| it.z_level);
        for draw in buffer.drain(..) {
            (draw.command)();
        }
    }
}

struct DrawCommand {
    z_level: i32,
    command: Box<dyn FnOnce() -> ()>,
}
