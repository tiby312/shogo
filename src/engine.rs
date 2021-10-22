macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Debug)]
pub enum Event {
    MouseDown([f64; 2]),
}

pub struct Engine {
    events: Rc<RefCell<Vec<Event>>>,
    buffer: Vec<Event>,
    last: f64,
    frame_rate: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct NoElem;

impl Engine {
    pub fn new(canvas: &str, frame_rate: usize) -> Result<Engine, NoElem> {
        let frame_rate = ((1.0 / frame_rate as f64) * 100.0).round() as usize;

        let events = Rc::new(RefCell::new(Vec::new()));

        let ee = events.clone();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas: web_sys::HtmlCanvasElement = document
            .get_element_by_id(canvas)
            .ok_or(NoElem)?
            .dyn_into()
            .unwrap();

        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            ee.borrow_mut()
                .push(Event::MouseDown([e.client_x() as f64, e.client_y() as f64]));
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);

        canvas.set_onclick(Some(&cb.as_ref().unchecked_ref()));
        //TODO dont leak.
        cb.forget();

        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

        Ok(Engine {
            events,
            buffer: Vec::new(),
            last: performance.now(),
            frame_rate,
        })
    }

    pub async fn next<'a>(&'a mut self) -> Result<&[Event],GameError> {
        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

        let tt = performance.now();
        let diff = performance.now() - self.last;

        if self.frame_rate as f64 - diff > 0.0 {
            let d = (self.frame_rate as f64 - diff) as usize;
            delay(d).await;
        }

        self.last = tt;
        {
            self.buffer.clear();
            let ee = &mut self.events.borrow_mut();
            self.buffer.append(ee);
            assert!(ee.is_empty());
        }

        Ok(&self.buffer)
    }
}

pub async fn delay(a: usize) {
    use std::convert::TryInto;

    let a: i32 = a.try_into().expect("can't delay with that large a value!");

    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, a)
            .unwrap();
    });

    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("timeout failed");
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
}

struct GameDelta {}

struct MoveUnpacker {}
impl MoveUnpacker {
    fn tick(&mut self) -> (bool, GameDelta) {
        unimplemented!();
    }
}

use ws_stream_wasm::*;

pub struct MyWebsocket {
    socket: ws_stream_wasm::WsStream,
}

impl MyWebsocket {
    pub async fn new(addr: &str) -> Result<MyWebsocket,GameError> {
        let socket_create = WsMeta::connect(addr, None);
        let (_, socket) = socket_create.await.map_err(|_|GameError::SocketErr)?;
        Ok(MyWebsocket { socket })
    }

    pub async fn send<T>(&mut self, a: T) ->Result<(),GameError> {
        use futures::SinkExt;
        unimplemented!();
    }

    pub async fn recv<T>(&mut self) ->Result<T,GameError> {
        use futures::StreamExt;
        unimplemented!();
    }
}

#[derive(Debug,Copy,Clone)]
pub enum GameError{
    SocketErr
}

pub async fn run_game()->Result<(),GameError> {
    
    let s1 = MyWebsocket::new("ws://127.0.0.1:3012");
    let s2 = MyWebsocket::new("ws://127.0.0.1:3012");

    let mut engine = Engine::new("canvas", 10).unwrap();
        
    let mut s1=s1.await?;
    let mut s2=s2.await?;

    let mut gamestate:GameState = s2.recv().await?;

    let mut move_acc = MovePacker {};
    loop {
        s1.send(move_acc.wrap()).await?;
        let mut unpacker: MoveUnpacker = s1.recv().await?;

        for _ in 0..60 {
            for event in engine.next().await?{
                match event {
                    &Event::MouseDown(mouse_pos) => {
                        move_acc.tick(mouse_pos);
                    }
                }
            }

            let (send_back_game, game_delta) = unpacker.tick();

            if send_back_game {
                s2.send(&gamestate).await?;
            }

            gamestate.tick(game_delta);

            //draw game state
        }
    }
}
