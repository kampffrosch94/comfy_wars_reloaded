use std::{
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
};

use base::*;

use macroquad::prelude::*;
use notify::{Event, INotifyWatcher, RecursiveMode, Watcher};

#[no_mangle]
pub unsafe extern "C" fn __cxa_thread_atexit_impl() {}

#[macroquad::main("MyGame")]
async fn main() {
    struct Sample;

    impl ContextTrait for Sample {
        fn draw_rect(&self, rect: base::Rect, c: base::Color) {
            let color = macroquad::prelude::Color {
                r: c.r,
                g: c.g,
                b: c.b,
                a: c.a,
            };
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
        }

        /// time since program start
        fn time(&self) -> f64 {
            get_time()
        }

        fn draw_text(&self, text: &str, x: f32, y: f32) {
            draw_text_ex(text, x, y, TextParams::default());
        }
    }

    let path = "../target/debug/libworker.so";
    let mut worker = WorkerReloader::new(path);

    loop {
        clear_background(BLACK);

        // let start = Instant::now();
        worker.update(&mut Sample {});
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
    persist_state: PersistWrapper,
}

struct WorkerWrapper {
    #[allow(unused)]
    lib: libloading::Library,
    #[allow(improper_ctypes_definitions)]
    update: extern "C" fn(&mut dyn ContextTrait, &mut PersistWrapper) -> (),
}

impl WorkerReloader {
    fn new(path: &str) -> Self {
        let path = PathBuf::from(path);
        let worker = Self::create_worker(&path);

        let (tx, receiver) = std::sync::mpsc::channel();

        let mut watcher = notify::recommended_watcher(tx).unwrap();
        watcher
            .watch(path.parent().unwrap(), RecursiveMode::NonRecursive)
            .unwrap();

        let create: libloading::Symbol<extern "C" fn() -> PersistWrapper> =
            unsafe { worker.lib.get(b"permanent_state").unwrap() };
        let persist_state = create();

        let worker = Some(worker);
        Self {
            worker,
            watcher,
            receiver,
            path,
            persist_state,
        }
    }

    fn create_worker(path: &Path) -> WorkerWrapper {
        unsafe {
            let lib = libloading::Library::new(path).unwrap();
            let symb: libloading::Symbol<
                extern "C" fn(&mut dyn ContextTrait, &mut PersistWrapper) -> (),
            > = lib.get(b"update").unwrap();
            let update: extern "C" fn(&mut dyn ContextTrait, &mut PersistWrapper) -> () =
                std::mem::transmute(symb.into_raw());
            WorkerWrapper { lib, update }
        }
    }

    fn update(&mut self, ctx: &mut dyn ContextTrait) {
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

        let update = self.worker.as_ref().unwrap().update;
        let ps = &mut self.persist_state;

	update(ctx, ps);
    }
}
