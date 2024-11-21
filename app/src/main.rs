use std::{path::Path, time::Instant};

use base::Context;

use macroquad::prelude::*;

#[macroquad::main("MyGame")]
async fn main() {
    let mut ctx = Context {
        draw: Box::new(|rect, c| {
            let color = macroquad::prelude::Color {
                r: c.r,
                g: c.g,
                b: c.b,
                a: c.a,
            };
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
        }),
        time: Box::new(|| get_time()),
    };

    loop {
        clear_background(BLACK);

        let start = Instant::now();
        call_dynamic(&mut ctx);
        let duration = start.elapsed();

        println!("Reload + Execution took: {:?}", duration);

        //draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        //draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);

        let s = format!("FPS: {}", get_fps());
        draw_text(&s, 20.0, 20.0, 30.0, WHITE);
        next_frame().await
    }
}

fn call_dynamic(ctx: &mut Context) {
    let path = "../target/debug/libworker.so";
    if !Path::new(&path).exists() {
        return;
    }
    unsafe {
        let lib = libloading::Library::new(path).unwrap();
        let symb: libloading::Symbol<extern "C" fn(&mut Context) -> ()> =
            lib.get(b"update").unwrap();
        let func: extern "C" fn(&mut Context) -> () = std::mem::transmute(symb.into_raw());
        func(ctx);
        lib.close().unwrap();
    }
}

struct WorkerReloader {
    worker: WorkerWrapper,
}

struct WorkerWrapper {
    lib: libloading::Library,
    update: extern "C" fn(&mut Context) -> (),
}

impl WorkerReloader {
    fn new(path: &str) -> Self {
        let worker = unsafe {
            let lib = libloading::Library::new(path).unwrap();
            let symb: libloading::Symbol<extern "C" fn(&mut Context) -> ()> =
                lib.get(b"update").unwrap();
            let update: extern "C" fn(&mut Context) -> () = std::mem::transmute(symb.into_raw());
            WorkerWrapper { lib, update }
        };

        Self { worker }
    }
}
