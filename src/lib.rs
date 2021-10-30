mod data;
mod engine;

use gloo::{
    events::EventListener,
    render::{request_animation_frame, AnimationFrame},
};
use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    AudioBuffer, AudioContext, Event, HtmlCanvasElement, KeyboardEvent, WebGlBuffer, WebGlProgram,
    WebGlRenderingContext, WebGlShader, WebGlTexture, WebGlUniformLocation,
};

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

pub async fn test_game() {
    console_log!("test game start!");
    let mut engine = engine::Engine::new(10).unwrap();

    let canvas = get_canvas("mycanvas");

    let ctx = get_context_2d(&canvas);

    engine.add_on_mouse_move(&canvas);

    let my_button=get_button("mybutton");
    engine.add_on_click(&my_button);

    let mut mouse_pos = [0.0; 2];

    while let Ok(events)=engine.next().await{
        for event in events {
            match event {
                engine::Event::MouseDown(elem, _) => {
                    if elem.id()=="mybutton"{
                        console_log!("button pushed!");   
                    }
                },
                engine::Event::MouseMove(elem,mouse_event)=>{
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
}

/*

struct MovePacker {}
impl MovePacker {
    fn tick(&mut self, a: [f64; 2]) {}
    fn wrap(&mut self) -> Vec<u8> {
        unimplemented!();
    }
}

struct GameState {}
impl GameState {
    fn tick(&mut self, a: GameDelta) {
        //update game state
    }
    pub fn draw(&self) -> Result<(), engine::GameError> {
        unimplemented!();
    }
}

struct GameDelta {}

struct MoveUnpacker {}
impl MoveUnpacker {
    fn tick(&mut self) -> (bool, GameDelta) {
        unimplemented!();
    }
}
pub async fn run_game()->Result<(),engine::GameError> {
    console_log!("YOYO");
    let frame_rate=60;
    let s1 = engine::MyWebsocket::new("ws://127.0.0.1:3012");
    let s2 = engine::MyWebsocket::new("ws://127.0.0.1:3012");

    let mut engine = engine::Engine::new("canvas", frame_rate).unwrap();

    let mut renderer=engine::Renderer::new(20);


    let mut s1=s1.await?;
    let mut s2=s2.await?;

    let mut gamestate:GameState = s2.recv().await?;

    let mut move_acc = MovePacker {};
    loop {

        s1.send(move_acc.wrap()).await?;
        let mut unpacker=s1.recv::<MoveUnpacker>().await?;

        for _ in 0..frame_rate {

            let dd=renderer.render(||gamestate.draw().unwrap());

            for event in engine.next().await?{
                match event {
                    &engine::Event::MouseDown(mouse_pos) => {
                        move_acc.tick(mouse_pos);
                    }
                }
            }

            let (send_back_game, game_delta) = unpacker.tick();

            if send_back_game {
                s2.send(&gamestate).await?;
            }

            dd.await?;

            gamestate.tick(game_delta);

        }
    }

}
*/

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    test_game().await;
    Ok(())
}
