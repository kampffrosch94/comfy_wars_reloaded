use std::time::{SystemTime, UNIX_EPOCH};

use base::{Color, Context, Rect};

#[no_mangle]
pub extern "C" fn update(c: &mut Context) {
    let start = SystemTime::now();
    (c.draw)(
        Rect {
            x: 80.,
            y: 50.,
            w: 500.0,
            h: 50.,
        },
        Color {
            r: 0.,
            g: 1.,
            b: 0.0,
            a: 1.0,
        },
    );

    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let time = (c.time)();
    let dx = ( time * 3.).sin() * 200.;
    (c.draw)(
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
