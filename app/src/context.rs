use std::cell::RefCell;

use base::*;

use macroquad::prelude::*;

use crate::util::texture_store::TextureStore;

#[derive(Default)]
pub struct Context {
    draw_buffer: RefCell<Vec<DrawCommand>>,
    pub textures: TextureStore,
}

impl ContextTrait for Context {
    /// time since program start
    fn time(&self) -> f64 {
        get_time()
    }

    fn draw_rect(&mut self, rect: base::Rect, c: base::Color, z_level: i32) {
        let color = macroquad::prelude::Color {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        };

        let command = move || {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
        };
        self.draw_buffer.borrow_mut().push(DrawCommand {
            z_level,
            command: Box::new(command),
        });
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32) {
        draw_text_ex(text, x, y, TextParams::default());
    }

    fn draw_texture(&mut self, name: &str, src: base::Rect, x: f32, y: f32, z_level: i32) {
        // load if not in texture store
        // then add to draw buffer
        if let Some(texture) = self.textures.get(name) {
            let source = Some(macroquad::math::Rect {
                x: src.x,
                y: src.y,
                w: src.w,
                h: src.h,
            });
            let params = DrawTextureParams {
                source,
                ..Default::default()
            };
            let command = move || {
                draw_texture_ex(&texture, x, y, WHITE, params);
            };
            self.draw_buffer.borrow_mut().push(DrawCommand {
                z_level,
                command: Box::new(command),
            });
        } else {
            self.draw_text(&format!("ERROR('{name}')"), x, y)
        }
    }
}

impl Context {
    /// executes deferred drawing, should be called once per frame
    pub fn process(&mut self) {
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
