use std::{collections::HashMap, ffi::c_void, panic::AssertUnwindSafe};

use base::{Color, ContextTrait, Key, PersistWrapper, Rect};
use cosync::CosyncInput;
use sprite::Sprite;
mod genarena;
mod sprite;

/// not dropped across reloads
struct PersistentState {
    sprites: HashMap<String, Sprite>,
}

impl PersistentState {
    fn new() -> Self {
        Self { sprites: sprite::load_sprites("../assets/sprites.json")}
    }
}

#[no_mangle]
pub extern "C" fn permanent_state() -> PersistWrapper {
    let state = PersistentState::new();
    let size = size_of_val(&state);
    let align = align_of_val(&state);
    let boxed = Box::new(state);
    let ptr = Box::into_raw(boxed) as *mut c_void;
    PersistWrapper { ptr, size, align }
}

/// dropped and recreated on reload
/// you can change this definition without breaking hotreloading
struct FleetingState {
    queue: cosync::Cosync<PersistentState>,
}

impl FleetingState {
    fn new() -> Self {
        let mut queue = cosync::Cosync::new();
        queue.queue(move |mut input: CosyncInput<PersistentState>| async move {
            for _ in 0..5 {
                cosync::sleep_ticks(30).await;
                let mut s = input.get();
                // s.number += 1;
            }
        });
        Self { queue }
    }
}

#[no_mangle]
pub extern "C" fn fleeting_state_create() -> PersistWrapper {
    let state = FleetingState::new();
    let size = size_of_val(&state);
    let align = align_of_val(&state);
    let boxed: Box<FleetingState> = Box::new(state);
    let ptr = Box::into_raw(boxed) as *mut c_void;
    PersistWrapper { ptr, size, align }
}

#[no_mangle]
pub extern "C" fn fleeting_state_dispose(pers: &mut PersistWrapper, fleet: PersistWrapper) {
    let ptr = fleet.ptr as *mut FleetingState;
    // put state into a box which gets dropped at the end of this method
    let mut boxed: Box<FleetingState> = unsafe { Box::from_raw(ptr) };
    boxed.queue.run_blocking(pers.ref_mut());
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn update(
    c: &mut dyn ContextTrait,
    persistent_state: &mut PersistWrapper,
    fleeting_state: &mut PersistWrapper,
) {
    _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let s: &mut PersistentState = persistent_state.ref_mut();
        update_inner(c, s, fleeting_state.ref_mut());
    }));
}

fn update_inner(c: &mut dyn ContextTrait, s: &mut PersistentState, f: &mut FleetingState) {
    f.queue.run_until_stall(s);

    //c.draw_text(&format!("Persistent Number: {}", s.number), 200., 200.);

    s.sprites["red_infantry"].draw(c, 50., 50., 1);

    if c.is_pressed(Key::MouseLeft) {
	s.sprites["arrow_s"].draw(c, 80., 50., 1);
    }

    c.draw_rect(
        Rect {
            x: 280.,
            y: 50.,
            w: 250.0,
            h: 50.,
        },
        Color {
            r: 0.,
            g: 1.,
            b: 0.0,
            a: 1.0,
        },
        0,
    );

    let r = Rect {
        x: 0.,
        y: 0.,
        w: 16. * 18.,
        h: 16. * 11.,
    };

    //c.draw_texture("tiles", r, 50., 50., -1);

    let dx = (c.time() * 2.).sin() * 200.;
    c.draw_rect(
        Rect {
            x: 300. + dx as f32,
            y: 500.,
            w: 200.0,
            h: 50.,
        },
        Color {
            r: 1.,
            g: 0.,
            b: 0.0,
            a: 1.0,
        },
        0,
    );

    //println!("Number: {}", dx);
}
