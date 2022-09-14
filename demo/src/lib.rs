use gloo_console::log;
use serde::{Deserialize, Serialize};
use shogo::utils;
use wasm_bindgen::prelude::*;

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

#[wasm_bindgen]
pub async fn main_entry() {
    use futures::StreamExt;

    log!("demo start");

    let (canvas, button, shutdown_button) = (
        utils::get_by_id_canvas("mycanvas"),
        utils::get_by_id_elem("mybutton"),
        utils::get_by_id_elem("shutdownbutton"),
    );

    let offscreen = canvas.transfer_control_to_offscreen().unwrap_throw();

    let (mut worker, mut response) = shogo::EngineMain::new(offscreen).await;

    let _handler = worker.register_event(&canvas, "mousemove", |e| {
        let [x, y] = convert_coord(e.elem, e.event);
        MEvent::CanvasMouseMove { x, y }
    });

    let _handler = worker.register_event(&button, "click", |_| MEvent::ButtonClick);

    let _handler = worker.register_event(&shutdown_button, "click", |_| MEvent::ShutdownClick);

    let _: () = response.next().await.unwrap_throw();
    log!("main thread is closing");
}

#[wasm_bindgen]
pub async fn worker_entry() {
    use shogo::simple2d;
    
    let (mut w, ss) = shogo::EngineWorker::new().await;
    let mut frame_timer = shogo::FrameTimer::new(30, ss);

    let canvas = w.canvas();

    let ctx = simple2d::CtxWrap::new(&utils::get_context_webgl2_offscreen(&canvas));

    let mut mouse_pos = [0.0f32; 2];

    let mut color_iter = COLORS.iter().cycle().peekable();

    ctx.setup_alpha();

    let mut vv=simple2d::VertSys::new();

    let walls=vv.create_static(&ctx.ctx,|rr|{
        rr.rect(simple2d::Rect {
            x: 40.0,
            y: 40.0,
            w: 800.0 - 80.0,
            h: 600.0 - 80.0,
        });
    }).unwrap();

    
    let (mut draw_sys, mut buffer) = (
        ctx.shader_system(),
        ctx.buffer_dynamic(),
    );

    'outer: loop {
        for e in frame_timer.next().await {
            match e {
                MEvent::CanvasMouseMove { x, y } => mouse_pos = [*x, *y],
                MEvent::ButtonClick => {
                    let _ = color_iter.next();
                }
                MEvent::ShutdownClick => break 'outer,
            }
        }

        let radius = 4.0;
        let game_dim = [canvas.width() as f32, canvas.height() as f32];

        vv.fill(&mut buffer,|verts|{
            verts.line(radius, mouse_pos, [0.0, 0.0]);
            verts.line(radius, mouse_pos, game_dim);
            verts.line(radius, mouse_pos, [0.0, game_dim[1]]);
            verts.line(radius, mouse_pos, [game_dim[0], 0.0]); 
        });

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        let mut v = draw_sys.view(game_dim, [0.0, 0.0]);

        v.draw_triangles(&walls, &[1.0, 1.0, 1.0, 0.2]);

        v.draw_triangles(&buffer, color_iter.peek().unwrap_throw());

        ctx.flush();
    }

    w.post_message(());

    log!("worker thread closing");
}

fn convert_coord(canvas: &web_sys::HtmlElement, event: &web_sys::Event) -> [f32; 2] {
    use wasm_bindgen::JsCast;
    shogo::simple2d::convert_coord(canvas, event.dyn_ref().unwrap_throw())
}
