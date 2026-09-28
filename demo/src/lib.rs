use std::borrow::Cow;

use futures::{FutureExt, SinkExt, channel::mpsc::UnboundedSender, select};
use gloo::console::log;
use serde::{Deserialize, Serialize};
use shogo::utils;
use wasm_bindgen::prelude::*;
use web_sys::{Event, EventTarget, MouseEvent};

const COLORS: &[[f32; 4]] = &[
    [1.0, 0.0, 0.0, 0.5],
    [0.0, 1.0, 0.0, 0.5],
    [0.0, 0.0, 1.0, 0.5],
];

///Common data sent from the main thread to the worker.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MEvent {
    CanvasMouseMove { x: f32, y: f32 },
    ButtonClick,
    ShutdownClick,
}

fn reg< S: Into<Cow<'static, str>>>(tx:&UnboundedSender<MEvent>,target:&EventTarget, event: S, mut handler: impl FnMut(&UnboundedSender<MEvent>,&Event) + 'static) ->gloo::events::EventListener{
    let tx=tx.clone();
    gloo::events::EventListener::new(target,event,move |e|{
        handler(&tx,e)
    })
}

#[wasm_bindgen]
pub async fn main_entry() {
    use futures::StreamExt;

    log!("demo start");

    let (canvas, button, shutdown_button) = (
        utils::get_by_id_canvas("mycanvas"),
        utils::get_by_id_elem("mybutton"),
        utils::get_by_id_elem("shutdownbutton"),
    );

    let ctx=canvas.get_context("2d").unwrap_throw().unwrap_throw();
    let ctx=ctx.dyn_ref::<web_sys::CanvasRenderingContext2d>().unwrap_throw();

    ctx.set_fill_style_str("red");
    ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    

    
    let (tx,mut rx)=futures::channel::mpsc::unbounded();
 

    let _e=[
        reg(&tx, &canvas, "mousemove", move |tx, e| {
            let e:&MouseEvent = e.dyn_ref().unwrap_throw();
            let x=e.x() as f32;
            let y = e.y() as f32;
            tx.unbounded_send(MEvent::CanvasMouseMove { x, y }).unwrap_throw();
        }),
        reg(&tx, &button, "click", move |tx, _| {
            tx.unbounded_send(MEvent::ButtonClick).unwrap_throw();
        }),
        reg(&tx, &shutdown_button, "click", move |tx, _| {
            tx.unbounded_send(MEvent::ShutdownClick).unwrap_throw();
        })
    ];
    


    let mut frame_timer = shogo::Timer::new(30);

    loop{
        loop{
            select!{
                _ = frame_timer.next().fuse() => {
                    break;
                }
                e = rx.next() => {
                    log!("received event: {}", format!("{:?}", e));
                }
            }
        }

        ctx.set_fill_style_str("white");
        draw_triangle(&ctx,[50.0,50.0]);


    }
}

fn draw_triangle(ctx: &web_sys::CanvasRenderingContext2d, center: [f64; 2]) {
    ctx.begin_path();
    ctx.move_to(center[0] - 25.0, center[1] + 25.0);
    ctx.line_to(center[0] + 25.0, center[1] + 25.0);
    ctx.line_to(center[0], center[1] - 25.0);
    ctx.close_path();
    ctx.fill();
}


// #[wasm_bindgen]
// pub async fn worker_entry() {
//     use shogo::simple2d;

//     let (mut w, ss) = shogo::EngineWorker::new().await;
//     let mut frame_timer = shogo::FrameTimer::new(30, ss);

//     let canvas = w.canvas();
//     let ctx = simple2d::ctx_wrap(&utils::get_context_webgl2_offscreen(&canvas));

//     //TODO put this in the library
//     ctx.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

//     let mut draw_sys = ctx.shader_system();
//     let mut buffer = ctx.buffer_dynamic();
//     let cache = &mut vec![];
//     simple2d::shapes(cache).rect(simple2d::Rect {
//         x: 40.0,
//         y: 40.0,
//         w: 800.0 - 80.0,
//         h: 600.0 - 80.0,
//     });
//     let walls = ctx.buffer_static_clear(cache);

//     ctx.setup_alpha();

//     // setup game data
//     let mut mouse_pos = [0.0f32; 2];
//     let mut color_iter = COLORS.iter().cycle().peekable();
//     let radius = 4.0;
//     let game_dim = [canvas.width() as f32, canvas.height() as f32];

//     'outer: loop {
//         for e in frame_timer.next().await {
//             match e {
//                 MEvent::CanvasMouseMove { x, y } => mouse_pos = [*x, *y],
//                 MEvent::ButtonClick => {
//                     let _ = color_iter.next();
//                 }
//                 MEvent::ShutdownClick => break 'outer,
//             }
//         }

//         simple2d::shapes(cache)
//             .line(radius, mouse_pos, [0.0, 0.0])
//             .line(radius, mouse_pos, game_dim)
//             .line(radius, mouse_pos, [0.0, game_dim[1]])
//             .line(radius, mouse_pos, [game_dim[0], 0.0]);

//         buffer.update_clear(cache);

//         ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

//         let matrix = projection(game_dim, [0.0, 0.0]);

//         let mut v = draw_sys.view(&matrix);
//         v.draw_triangles(&walls, &[1.0, 1.0, 1.0, 0.2]);
//         v.draw_triangles(&buffer, color_iter.peek().unwrap_throw());

//         ctx.flush();
//     }

//     w.post_message(());

//     log!("worker thread closing");
// }

// fn convert_coord(canvas: &web_sys::HtmlElement, event: &web_sys::Event) -> [f32; 2] {
//     use wasm_bindgen::JsCast;
//     shogo::simple2d::convert_coord(canvas, event.dyn_ref().unwrap_throw())
// }

// fn projection(dim: [f32; 2], offset: [f32; 2]) -> [f32; 16] {
//     let scale = |scalex, scaley, scalez| {
//         [
//             scalex, 0., 0., 0., 0., scaley, 0., 0., 0., 0., scalez, 0., 0., 0., 0., 1.0,
//         ]
//     };

//     let translation = |tx, ty, tz| {
//         [
//             1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., tx, ty, tz, 1.,
//         ]
//     };

//     let x_rotation = |angle_rad: f32| {
//         let c = angle_rad.cos();
//         let s = angle_rad.sin();

//         [1., 0., 0., 0., 0., c, s, 0., 0., -s, c, 0., 0., 0., 0., 1.]
//     };

//     let y_rotation = |angle_rad: f32| {
//         let c = angle_rad.cos();
//         let s = angle_rad.sin();

//         [c, 0., -s, 0., 0., 1., 0., 0., s, 0., c, 0., 0., 0., 0., 1.]
//     };

//     let z_rotation = |angle_rad: f32| {
//         let c = angle_rad.cos();
//         let s = angle_rad.sin();

//         [c, s, 0., 0., -s, c, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.]
//     };

//     use webgl_matrix::prelude::*;

//     let mut id = Mat4::identity();

//     let az = &translation(-dim[0] / 2. + offset[0], -dim[1] / 2. + offset[1], 0.0);
//     let t = &translation(-1.0, 1.0, 0.0);
//     let a1 = &scale(2.0, -2.0, 0.0);
//     let a2 = &scale(1.0 / dim[0], 1.0 / dim[1], 0.0);
//     id.mul(az).mul(a1).mul(a2).mul(&t);
//     id
// }
