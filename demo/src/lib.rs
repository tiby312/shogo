use futures::future::FutureExt;
use futures::stream::StreamExt;
use gloo_console::log;
use wasm_bindgen::{prelude::*, JsCast};
use wengine::{GameE, GameEvent};

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    log!("hello there");

    let document = gloo::utils::document();
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

    let mut engine = wengine::engine(60);

    let (sender, mut receiver) = futures::channel::mpsc::unbounded();

    let _click = wengine::register_click(sender.clone(), &button);
    let _mouse = wengine::register_mousemove(sender.clone(), &canvas);

    let mut mouse_pos = [0.0; 2];

    let mut color_iter = ["black", "red", "green"].into_iter().cycle();
    let mut current_color = color_iter.next().unwrap_throw();

    loop {
        loop {
            futures::select!(
                GameE{element,event} = receiver.next().map(|x|x.unwrap()) =>{
                    match event{
                        GameEvent::MouseClick(_mouse)=>{
                            if element == button {
                                current_color = color_iter.next().unwrap_throw();
                            }
                        },
                        GameEvent::MouseMove(mouse)=>{
                            if element == *canvas.as_ref() {
                                mouse_pos = convert_coord(element, mouse);
                            }
                        }
                    }
                },
                () = engine.next().fuse() =>{
                    break;
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
