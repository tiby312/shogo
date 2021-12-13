use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn convert_coord(canvas: web_sys::HtmlElement, e: web_sys::MouseEvent) -> [f64; 2] {
    let [x, y] = [e.client_x() as f64, e.client_y() as f64];
    let bb = canvas.get_bounding_client_rect();
    let tl = bb.x();
    let tr = bb.y();
    [x - tl, y - tr]
}

pub struct ElemSet {
    pub window: web_sys::Window,
    pub document: web_sys::Document,
    pub canvas: web_sys::HtmlCanvasElement,
    pub ctx: web_sys::CanvasRenderingContext2d,
    pub my_button: web_sys::HtmlElement,
}
impl ElemSet {
    fn new() -> Result<ElemSet, JsValue> {
        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let canvas: web_sys::HtmlCanvasElement = document
            .get_element_by_id("mycanvas")
            .unwrap_throw()
            .dyn_into()?;

        let ctx: web_sys::CanvasRenderingContext2d =
            canvas.get_context("2d")?.unwrap_throw().dyn_into()?;

        let my_button: web_sys::HtmlElement = document
            .get_element_by_id("mybutton")
            .unwrap_throw()
            .dyn_into()?;

        Ok(ElemSet {
            window,
            document,
            canvas,
            ctx,
            my_button,
        })
    }
}

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    console_log!("test game start!");
    let mut engine = wengine::Engine::new(60);

    let ElemSet {
        canvas,
        ctx,
        my_button,
        ..
    } = ElemSet::new()?;

    engine.add_on_mouse_move(&canvas);

    engine.add_on_click(&my_button);

    let mut mouse_pos = [0.0; 2];

    while let Some(events) = engine.next().await {
        for event in events {
            match event {
                wengine::Event::MouseDown(elem, _) => {
                    if elem == my_button {
                        console_log!("button pushed!");
                    }
                }
                wengine::Event::MouseMove(elem, mouse_event) => {
                    if elem.id() == "mycanvas" {
                        let pos = convert_coord(elem, mouse_event);
                        console_log!("mouse pos={:?}", pos);
                        mouse_pos = pos;
                    }
                }
            }
        }

        console_log!("clearing");

        ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        ctx.fill_rect(0.0, 0.0, mouse_pos[0], mouse_pos[1]);
    }

    Ok(())
}
