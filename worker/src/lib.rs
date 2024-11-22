use std::{ffi::c_void, panic::AssertUnwindSafe};

use base::{Color, ContextTrait, PersistWrapper, Rect};

struct MyState {
    strings: Vec<((f32, f32), String)>,
}

#[no_mangle]
pub extern "C" fn permanent_state() -> PersistWrapper {
    let state = MyState {
        strings: Vec::new(),
    };
    let size = size_of_val(&state);
    let align = align_of_val(&state);
    let boxed = Box::new(state);
    let ptr = Box::into_raw(boxed) as *mut c_void;
    PersistWrapper { ptr, size, align }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn update(c: &mut dyn ContextTrait, ps: &mut PersistWrapper) {
    let s: &mut MyState = ps.ref_mut();
    _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
        update_inner(c, s);
    }));
}

fn update_inner(c: &mut dyn ContextTrait, s: &mut MyState) {
    let len = s.strings.len();
    // s.strings.clear();
    if len < 3 {
        let pos = (20., 150. + len as f32 * 20.);
        let new = "New Text".into();
        s.strings.push((pos, new));
    }

    for (pos, s) in &s.strings {
        c.draw_text(s, pos.0, pos.1);
    }

    c.draw_text(&s.strings[2].1, 40., 302.);

    c.draw_rect(
        Rect {
            x: 80.,
            y: 50.,
            w: 500.0,
            h: 50.,
        },
        Color {
            r: 1.,
            g: 1.,
            b: 0.0,
            a: 1.0,
        },
    );

    let time = c.time();
    let dx = (time * 2.).sin() * 200.;
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
    );

    //println!("Number: {}", dx);
}
