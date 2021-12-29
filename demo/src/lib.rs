use gloo::console::log;
use wasm_bindgen::prelude::*;

use shogo::{
    dots::{CtxExt, Shapes},
    utils,
};

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

    let mut worker =
        shogo::main::WorkerInterface::new(canvas.transfer_control_to_offscreen().unwrap_throw())
            .await;
    let _handler = worker.register_mousemove(&canvas);
    let _handler = worker.register_click(&button);
    let _handler = worker.register_click(&shutdown_button);

    worker.join().await;
    log!("main thread closing");
}

#[wasm_bindgen]
pub async fn worker_entry() {
    let mut w = shogo::worker::WorkerHandler::new(30).await;

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
            match e {
                &shogo::MEvent::MouseMove { elem, x, y, .. } => match elem.as_str() {
                    "mycanvas" => {
                        mouse_pos = [x, y];
                    }
                    _ => {}
                },
                &shogo::MEvent::MouseClick { elem, .. } => match elem.as_str() {
                    "mybutton" => {
                        let _ = color_iter.next();
                    }
                    "shutdownbutton" => {
                        break 'outer;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

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
