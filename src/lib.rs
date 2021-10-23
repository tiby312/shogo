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
    pub fn draw(&self)->Result<(),engine::GameError>{
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


#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    run_game().await;
    Ok(())
}
