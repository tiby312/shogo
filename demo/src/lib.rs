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

/*
///Call from worker.
pub fn register_click(foo:F,elem:&HtmlElement)
{

    let scope:web_sys::DedicatedWorkerGlobalScope =js_sys::global().dyn_into().unwrap_throw();

    let foo=Closure::once_into_js(foo);

    let k:&js_sys::Object=foo.dyn_ref().unwrap_throw();

    log!(k.to_string());

    let arr=js_sys::Array::new_with_length(1);
    arr.set(0,foo);

    let blob=web_sys::Blob::new_with_buffer_source_sequence(&arr).unwrap_throw();

    let s:js_sys::JsString=blob.to_string();
    log!("logged closure");
    scope.post_message(&s).unwrap_throw();


}
*/

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
    loop {
        for e in w.next().await {
            match e {
                &shogo::MEvent::MouseMove {
                    elem,
                    client_x,
                    client_y,
                } => match elem.as_str() {
                    "mycanvas" => {
                        mouse_pos = [client_x, client_y];
                    }
                    _ => {}
                },
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
}
use gloo::timers::future::TimeoutFuture;

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
    let _handler = worker.register_mousemove_handler(&canvas);

    TimeoutFuture::new(100000).await;
}

fn convert_coord(canvas: &web_sys::HtmlElement, e: web_sys::MouseEvent) -> [f32; 2] {
    let [x, y] = [e.client_x() as f32, e.client_y() as f32];
    let bb = canvas.get_bounding_client_rect();
    let tl = bb.x() as f32;
    let tr = bb.y() as f32;
    [x - tl, y - tr]
}
