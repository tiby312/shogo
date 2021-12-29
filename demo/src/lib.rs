use gloo::console::console_dbg;
use gloo::console::log;
use serde::{Deserialize, Serialize};
use shogo::{
    dots::{CtxExt, Shapes},
    utils,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MEvent {
    MouseMove { elem: String, x: f32, y: f32 },
    MouseClick { elem: String },
}

#[wasm_bindgen(start)]
pub async fn init_module() {
    log!("initing a module");
}

#[wasm_bindgen]
pub async fn main_entry() {
    log!("demo start!");

    let (canvas, button, shutdown_button) = (
        utils::get_by_id_canvas("mycanvas"),
        utils::get_by_id_elem("mybutton"),
        utils::get_by_id_elem("shutdownbutton"),
    );

    let offscreen = canvas.transfer_control_to_offscreen().unwrap_throw();

    let mut worker = shogo::main::WorkerInterface::new(offscreen).await;

    let _handler = worker.register_event(&canvas, "mousemove", |elem, event, _| {
        let [a, b] = convert_coord(&elem, event);
        MEvent::MouseMove {
            elem: elem.id(),
            x: a,
            y: b,
        }
    });

    let _handler = worker.register_event(&button, "click", |elem, _, _| MEvent::MouseClick {
        elem: elem.id(),
    });

    let _handler = worker.register_event(&shutdown_button, "click", |elem, _, _| {
        MEvent::MouseClick { elem: elem.id() }
    });

    worker.join().await;
    log!("main thread closing");
}

#[wasm_bindgen]
pub async fn worker_entry() {
    let mut w = shogo::worker::WorkerHandler::<MEvent>::new(30).await;

    let canvas = w.canvas();

    let ctx = utils::get_context_webgl2_offscreen(&canvas);

    let mut mouse_pos = [0.0f32; 2];

    let mut color_iter = {
        let colors = [
            [1.0, 0.0, 0.0, 0.5],
            [0.0, 1.0, 0.0, 0.5],
            [0.0, 0.0, 1.0, 0.5],
        ];
        colors.into_iter().cycle().peekable()
    };

    let (mut draw_sys, mut buffer, walls) = (
        ctx.shader_system(),
        ctx.buffer_dynamic(),
        ctx.buffer_static(vec![].rect(30.0, [40.0, 40.0], [800.0 - 80.0, 600.0 - 80.0])),
    );

    let mut verts = vec![];
    'outer: loop {
        for e in w.next().await {
            console_dbg!(e);
            match e {
                MEvent::MouseMove { elem, x, y } => match elem.as_str() {
                    "mycanvas" => {
                        log!("new mouse pos!!!!");
                        mouse_pos = [*x, *y];
                    }
                    _ => {}
                },
                MEvent::MouseClick { elem } => match elem.as_str() {
                    "mybutton" => {
                        let _ = color_iter.next();
                    }
                    "shutdownbutton" => {
                        break 'outer;
                    }
                    _ => {}
                },
            }
        }
        log!("worker:next frame!");

        let radius = 30.0;
        let game_dim = [canvas.width() as f32, canvas.height() as f32];

        verts.clear();
        verts.line(radius, mouse_pos, [0.0, 0.0]);
        verts.line(radius, mouse_pos, game_dim);
        verts.line(radius, mouse_pos, [0.0, game_dim[1]]);
        verts.line(radius, mouse_pos, [game_dim[0], 0.0]);
        buffer.update(&verts);

        ctx.clear_color(0.13, 0.13, 0.13, 1.0);
        ctx.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

        draw_sys.draw_circles(
            &buffer,
            game_dim,
            color_iter.peek().unwrap_throw(),
            [0.0, 0.0],
            radius,
        );
        draw_sys.draw_squares(&walls, game_dim, &[1.0, 1.0, 1.0, 0.2], [0.0, 0.0], radius);
    }
    log!("worker thread closing");
}

fn convert_coord(canvas: &web_sys::HtmlElement, event: &web_sys::Event) -> [f32; 2] {
    let e = event
        .dyn_ref::<web_sys::MouseEvent>()
        .unwrap_throw()
        .clone();

    let [x, y] = [e.client_x() as f32, e.client_y() as f32];
    let bb = canvas.get_bounding_client_rect();
    let tl = bb.x() as f32;
    let tr = bb.y() as f32;
    [x - tl, y - tr]
}
