use gloo_console::log;
use wasm_bindgen::{prelude::*, JsCast};

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    log!("hello there");

    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();
    let canvas: web_sys::HtmlCanvasElement = document
        .get_element_by_id("mycanvas")
        .unwrap_throw()
        .dyn_into()?;

    let ctx: web_sys::CanvasRenderingContext2d =
        canvas.get_context("2d")?.unwrap_throw().dyn_into()?;

    let button: web_sys::HtmlElement = document
        .get_element_by_id("mybutton")
        .unwrap_throw()
        .dyn_into()?;

    let mut engine = wengine::engine(30);

    engine.add_on_mouse_move(&canvas);

    engine.add_on_click(&button);

    let mut mouse_pos = [0.0; 2];

    let mut color_iter = ["black", "red", "green"].into_iter().cycle();
    let mut current_color = color_iter.next().unwrap_throw();

    while let Some(events) = engine.next().await {
        for event in events {
            match event {
                wengine::Event::MouseDown(elem, _) => {
                    if elem == button {
                        current_color = color_iter.next().unwrap_throw();
                    }
                }
                wengine::Event::MouseMove(elem, mouse_event) => {
                    if elem == *canvas.as_ref() {
                        mouse_pos = convert_coord(elem, mouse_event);
                    }
                }
            }
        }

        ctx.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());

        ctx.set_fill_style(&current_color.into());

        ctx.fill_rect(0.0, 0.0, mouse_pos[0], mouse_pos[1]);
    }

    Ok(())
}

fn convert_coord(canvas: web_sys::HtmlElement, e: web_sys::MouseEvent) -> [f64; 2] {
    let [x, y] = [e.client_x() as f64, e.client_y() as f64];
    let bb = canvas.get_bounding_client_rect();
    let tl = bb.x();
    let tr = bb.y();
    [x - tl, y - tr]
}
