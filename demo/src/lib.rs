use gloo::console::log;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn start() {
    log!("demo start!");

    let canvas = shogo::utils::get_canvas_by_id("mycanvas");
    let ctx = shogo::utils::get_context(&canvas, "2d");
    let button = shogo::utils::get_element_by_id("mybutton");
    let shutdown_button = shogo::utils::get_element_by_id("shutdownbutton");

    let mut engine = shogo::engine(60);

    let _handle = engine.add_mousemove(&canvas);
    let _handle = engine.add_click(&button);
    let _handle = engine.add_click(&shutdown_button);

    let mut mouse_pos = [0.0; 2];
    let mut color_iter = ["black", "red", "green"].into_iter().cycle();
    let mut current_color = color_iter.next().unwrap_throw();

    'outer: loop {
        
        for res in engine.get_last_delta().events {
            match res.event {
                shogo::Event::MouseClick(_mouse) => {
                    if res.element == button {
                        current_color = color_iter.next().unwrap_throw();
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

        let _handle = shogo::utils::render::request_animation_frame(|_| {
            ctx.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());

            ctx.set_fill_style(&current_color.into());

            ctx.fill_rect(0.0, 0.0, mouse_pos[0], mouse_pos[1]);
        });

        engine.next().await;

    }

    ctx.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());

    log!("all done!");
}

fn convert_coord(canvas: web_sys::HtmlElement, e: web_sys::MouseEvent) -> [f64; 2] {
    let [x, y] = [e.client_x() as f64, e.client_y() as f64];
    let bb = canvas.get_bounding_client_rect();
    let tl = bb.x();
    let tr = bb.y();
    [x - tl, y - tr]
}
