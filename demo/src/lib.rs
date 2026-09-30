use std::{borrow::Cow, cell::RefCell, rc::Rc};

use futures::{FutureExt, SinkExt, channel::mpsc::{UnboundedReceiver, UnboundedSender}, select};
use glam::{DVec2, Vec2};
use gloo::{console::log, net::http::Request};
use serde::{Deserialize, Serialize};
use shogo::utils;
use wasm_bindgen::prelude::*;
use web_sys::{Event, EventTarget, MouseEvent};
use gloo::render::{request_animation_frame, AnimationFrame};

const COLORS: &[[f32; 4]] = &[
    [1.0, 0.0, 0.0, 0.5],
    [0.0, 1.0, 0.0, 0.5],
    [0.0, 0.0, 1.0, 0.5],
];

///Common data sent from the main thread to the worker.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MEvent {
    CanvasMouseMove { x: f64, y: f64 },
    ButtonClick,
    ShutdownClick,
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

    

    
    let (tx,mut rx)=futures::channel::mpsc::unbounded();
 

    let _e=[
        shogo::reg(&tx, &canvas, "mousemove", move | e| {
            let e:&MouseEvent = e.dyn_ref().unwrap_throw();
            let x=e.x() as f64;
            let y = e.y() as f64;
            Some(MEvent::CanvasMouseMove { x, y })
        }),
        shogo::reg(&tx, &button, "click", move | _| {
            Some(MEvent::ButtonClick)
        }),
        shogo::reg(&tx, &shutdown_button, "click", move | _| {
            Some(MEvent::ShutdownClick)
        })
    ];
    



    let mut ship_pos=DVec2::new(50.0, 50.0);
    let mut rotation=0.0;
    let mut mouse_pos=DVec2::new(0.0, 0.0);





    let mut rr=shogo::RequestAnimationFrameMan::new();
    rr.request_animation_frame();
  

    loop{
        let dt=loop{
            select!{
                timestamp = rr.next().fuse() => {
                    log!("received frame delta: {}", timestamp.delta());
                    break timestamp.delta();
                }
                e = rx.next() => {
                    log!("received event: {}", format!("{:?}", e));
                    match e.unwrap_throw()
                    {
                        MEvent::CanvasMouseMove { x, y } => {
                            mouse_pos=DVec2::new(x , y );
                        },
                        MEvent::ButtonClick => {
                            // Handle button click event if needed
                        },
                        MEvent::ShutdownClick => {
                            // Handle shutdown click event if needed
                        }
                    }
                }
            }
        };

        let offset = mouse_pos-ship_pos;

        ship_pos = ship_pos + offset * 0.001 * dt;
        
        let rotation = offset.y.atan2(offset.x);

        ctx.set_fill_style_str("gray");
        ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    

        ctx.set_fill_style_str("white");
        draw_triangle(&ctx,ship_pos,rotation);

        
        rr.request_animation_frame();

      


    }
}

fn draw_triangle(ctx: &web_sys::CanvasRenderingContext2d, center: DVec2, rotation: f64) {
    ctx.save();
    ctx.translate(center.x , center.y).unwrap_throw();
    ctx.rotate(rotation).unwrap_throw();
    ctx.begin_path();
    ctx.move_to(-25.0, 25.0);
    ctx.line_to(25.0, 25.0);
    ctx.line_to(0.0, -25.0);
    ctx.close_path();
    ctx.fill();
    ctx.restore();
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
