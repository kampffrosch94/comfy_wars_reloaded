use std::{
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
};

use base::Context;

use macroquad::prelude::*;
use notify::{Event, INotifyWatcher, RecursiveMode, Watcher};

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

    let path = "../target/debug/libworker.so";
    let mut worker = WorkerReloader::new(path);

    loop {
        clear_background(BLACK);

        // let start = Instant::now();
        worker.update(&mut ctx);
        // let duration = start.elapsed();
        // println!("Reload + Execution took: {:?}", duration)));

        //draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        //draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);

        let fps = get_fps();
        let s = format!("FPS: {}", if fps > 55 && fps < 65 { 60 } else { fps });
        draw_text(&s, 20.0, 20.0, 30.0, WHITE);
        next_frame().await
    }
}

struct WorkerReloader {
    worker: Option<WorkerWrapper>,
    receiver: Receiver<Result<Event, notify::Error>>,
    path: PathBuf,
    #[allow(unused)]
    watcher: INotifyWatcher,
}

struct WorkerWrapper {
    #[allow(unused)]
    lib: libloading::Library,
    update: extern "C" fn(&mut Context) -> (),
}

impl WorkerReloader {
    fn new(path: &str) -> Self {
        let path = PathBuf::from(path);
        let worker = Some(Self::create_worker(&path));

        let (tx, receiver) = std::sync::mpsc::channel();

        let mut watcher = notify::recommended_watcher(tx).unwrap();
        watcher
            .watch(path.parent().unwrap(), RecursiveMode::NonRecursive)
            .unwrap();
        Self {
            worker,
            watcher,
            receiver,
            path,
        }
    }

    fn create_worker(path: &Path) -> WorkerWrapper {
        unsafe {
            let lib = libloading::Library::new(path).unwrap();
            let symb: libloading::Symbol<extern "C" fn(&mut Context) -> ()> =
                lib.get(b"update").unwrap();
            let update: extern "C" fn(&mut Context) -> () = std::mem::transmute(symb.into_raw());
            WorkerWrapper { lib, update }
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        let mut modified = false; // debounce reloading twice on multiple events
        while let Ok(event) = self.receiver.try_recv() {
            if let Ok(e) = event {
                if e.kind.is_create()
                    && e.paths
                        .iter()
                        .any(|p| p.file_name() == self.path.file_name())
                {
                    dbg!(&e);
                    modified = true;
                }
            }
        }

        if modified && Path::new(&self.path).exists() {
            self.worker = None; // need to unload before we can reload
            println!("Reloading!");
            self.worker = Some(Self::create_worker(&self.path));
        }

        (self.worker.as_ref().unwrap().update)(ctx);
    }
}
