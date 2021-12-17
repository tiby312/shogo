use futures::FutureExt;
use gloo::console::log;
use wasm_bindgen::prelude::*;
use wengine::utils;

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    log!("hello there");

    let canvas = utils::get_canvas_by_id("mycanvas");
    let ctx = utils::get_context(&canvas, "2d");
    let button = utils::get_element_by_id("mybutton");

    let mut frame_engine = wengine::frame_engine(60);
    let mut event_engine = wengine::event_engine();

    let _click = event_engine.register_click(&button);
    let _mouse = event_engine.register_mousemove(&canvas);

    let mut mouse_pos = [0.0; 2];

    let mut color_iter = ["black", "red", "green"].into_iter().cycle();
    let mut current_color = color_iter.next().unwrap_throw();

    loop {
        loop {
            futures::select_biased!(
                () = frame_engine.next().fuse() =>{
                    break;
                },
                wengine::EventElem{element,event} = event_engine.next().fuse() =>{
                    match event{
                        wengine::Event::MouseClick(_mouse)=>{
                            if element == button {
                                current_color = color_iter.next().unwrap_throw();
                            }
                        },
                        wengine::Event::MouseMove(mouse)=>{
                            if element == *canvas.as_ref() {
                                mouse_pos = convert_coord(element, mouse);
                            }
                        }
                    }
                }

            )
        }

        ctx.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());

        ctx.set_fill_style(&current_color.into());

        ctx.fill_rect(0.0, 0.0, mouse_pos[0], mouse_pos[1]);
    }
}

fn convert_coord(canvas: web_sys::HtmlElement, e: web_sys::MouseEvent) -> [f64; 2] {
    let [x, y] = [e.client_x() as f64, e.client_y() as f64];
    let bb = canvas.get_bounding_client_rect();
    let tl = bb.x();
    let tr = bb.y();
    [x - tl, y - tr]
}
