use gloo::console::log;
use wasm_bindgen::prelude::*;

use shogo::{
    dots::{CtxExt, Shapes},
    utils,
};

#[wasm_bindgen(start)]
pub async fn start() {
    log!("demo start!");

    let (canvas, button, shutdown_button) = (
        utils::get_by_id_canvas("mycanvas"),
        utils::get_by_id_elem("mybutton"),
        utils::get_by_id_elem("shutdownbutton"),
    );

    let ctx = utils::get_context_webgl2(&canvas);

    let mut engine = shogo::engine(60);

    let _handle = engine.add_mousemove(&canvas);
    let _handle = engine.add_click(&button);
    let _handle = engine.add_click(&shutdown_button);

    let mut mouse_pos = [0.0f32; 2];
    let mut color_iter = [
        [1.0, 0.0, 0.0, 0.5],
        [0.0, 1.0, 0.0, 0.5],
        [0.0, 0.0, 1.0, 0.5],
    ]
    .into_iter()
    .cycle()
    .peekable();

    let mut draw_sys = ctx.shader_system();
    let mut buffer = ctx.buffer_dynamic();
    let walls = ctx.buffer_static(vec![].rect(30.0, [40.0, 40.0], [800.0 - 80.0, 600.0 - 80.0]));

    let mut verts = vec![];
    'outer: loop {
        for res in engine.next().await.events {
            match res.event {
                shogo::Event::MouseClick(_mouse) => {
                    if res.element == button {
                        let _ = color_iter.next().unwrap_throw();
                    } else if res.element == shutdown_button {
                        break 'outer;
                    }
                }
                shogo::Event::MouseMove(mouse) => {
                    if res.element == *canvas.as_ref() {
                        mouse_pos = convert_coord(res.element, mouse);
                    }
                }
            }
        }

        let radius = 30.0;
        let game_dim = [canvas.width() as f32, canvas.height() as f32];

        verts.clear();
        verts
            .line(radius, mouse_pos, [0.0, 0.0])
            .line(radius, mouse_pos, game_dim)
            .line(radius, mouse_pos, [0.0, game_dim[1]])
            .line(radius, mouse_pos, [game_dim[0], 0.0]);

        buffer.update(&verts);

        ctx.clear_color(0.13, 0.13, 0.13, 1.0);
        ctx.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

        draw_sys.draw_circles(
            &buffer,
            game_dim,
            color_iter.peek().unwrap_throw(),
            &[0.0, 0.0],
            radius,
        );
        draw_sys.draw_squares(&walls, game_dim, &[1.0, 1.0, 1.0, 0.2], &[0.0, 0.0], radius);
    }

    log!("all done!");
}

fn convert_coord(canvas: web_sys::HtmlElement, e: web_sys::MouseEvent) -> [f32; 2] {
    let [x, y] = [e.client_x() as f32, e.client_y() as f32];
    let bb = canvas.get_bounding_client_rect();
    let tl = bb.x() as f32;
    let tr = bb.y() as f32;
    [x - tl, y - tr]
}
