use std::{ffi::c_void, iter::Enumerate};

use base::{Color, ContextTrait, PersistWraper, Rect};

struct MyState {
    strings: Vec<((f32, f32), String)>,
    complex: MyComplexType,
}

#[no_mangle]
pub extern "C" fn permanent_state() -> PersistWraper {
    let state = MyState {
        strings: Vec::new(),
        complex: MyComplexType {
            name: "Satou".into(),
            age: 22,
            occupation: Occupation::Hikikomori,
        },
    };
    let size = size_of_val(&state);
    let align = align_of_val(&state);
    let boxed = Box::new(state);
    let ptr = Box::into_raw(boxed) as *mut c_void;
    PersistWraper { ptr, size, align }
}

#[derive(Debug)] 
enum Occupation {
    Hikikomori,
    Student,
    Freeta,
}

#[derive(Debug)] 
struct MyComplexType {
    name: String,
    age: usize,
    occupation: Occupation,
}


#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn update(c: &mut dyn ContextTrait, ps: &mut PersistWraper) {
    let s: &mut MyState = ps.ref_mut();

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

    //println!("{}", &s.complex.name);
    // println!("Test");

    let formatted = format!("Dbg: {:?}", &s.complex);
    c.draw_text(&formatted, 30., 450.);

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
            b: 1.0,
            a: 1.0,
        },
    );

    let time = c.time();
    let dx = (time * 0.).sin() * 200.;
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
