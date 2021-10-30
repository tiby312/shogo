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


fn get_canvas(name: &str) -> web_sys::HtmlCanvasElement {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    document
        .get_element_by_id(name)
        .unwrap()
        .dyn_into()
        .unwrap()
}
fn get_context_2d(canvas: &web_sys::HtmlCanvasElement) -> web_sys::CanvasRenderingContext2d {
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}

fn get_button(name:&str)->web_sys::HtmlElement{
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    document
        .get_element_by_id(name)
        .unwrap()
        .dyn_into()
        .unwrap() 
}

fn convert_coord(canvas:&web_sys::HtmlElement,e:&web_sys::MouseEvent)->[f64;2]{
    let [x,y]=[e.client_x() as f64, e.client_y() as f64];
    let bb = canvas.get_bounding_client_rect();
    let tl = bb.x();
    let tr = bb.y();
    [x - tl, y - tr]
}


#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    console_log!("test game start!");
    let mut engine = wengine::Engine::new(60);

    let canvas = get_canvas("mycanvas");

    let ctx = get_context_2d(&canvas);

    engine.add_on_mouse_move(&canvas);

    let my_button=get_button("mybutton");
    engine.add_on_click(&my_button);

    let mut mouse_pos = [0.0; 2];

    while let Some(events)=engine.next().await{
        for event in events {
            match event {
                wengine::Event::MouseDown(elem, _) => {
                    if elem.id()=="mybutton"{
                        console_log!("button pushed!");   
                    }
                },
                wengine::Event::MouseMove(elem,mouse_event)=>{
                    if elem.id()=="mycanvas"{
                        let pos=convert_coord(elem,mouse_event);
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
