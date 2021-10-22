
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}



use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub enum Event{
    MouseDown([f64;2])
}


pub struct Engine{
    events:Rc<RefCell<Vec<Event>>>,
    buffer:Vec<Event>,
    last:f64,
    frame_rate:usize
}

#[derive(Debug,Copy,Clone)]
pub struct NoElem;

impl Engine{
    pub fn new(canvas:&str,frame_rate:usize)->Result<Engine,NoElem>{
        let frame_rate=((1.0 / frame_rate as f64) * 100.0).round() as usize;

        let events=Rc::new(RefCell::new(Vec::new()));

        let ee=events.clone();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();    
        let canvas: web_sys::HtmlCanvasElement = document.get_element_by_id(canvas).ok_or(NoElem)?.dyn_into().unwrap();
    
        let cb = Closure::wrap(Box::new(move |e:web_sys::MouseEvent| {
            ee.borrow_mut().push(Event::MouseDown([e.client_x() as f64,e.client_y() as f64]));
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);

        canvas.set_onclick(Some(&cb.as_ref().unchecked_ref()));
        //TODO dont leak.
        cb.forget();

        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");


        Ok(Engine{
            events,
            buffer:Vec::new(),
            last:performance.now(),
            frame_rate
        })
    }

    pub async fn next<'a>(&'a mut self)->Option<&[Event]>{
        
        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");


        let tt=performance.now();
        let diff=performance.now()-self.last;
      
        if self.frame_rate as f64-diff>0.0{
            let d=(self.frame_rate as f64-diff) as usize;
            delay(d).await;
        }
        
        self.last=tt;
        {
            self.buffer.clear();
            let ee=&mut self.events.borrow_mut();
            self.buffer.append(ee);
            assert!(ee.is_empty());
        }
        Some(&self.buffer)
    }
}





pub async fn delay(a:usize){
    use std::convert::TryInto;

    let a:i32=a.try_into().expect("can't delay with that large a value!");

    let promise=js_sys::Promise::new(&mut |resolve,_|{
        web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(&resolve,a).unwrap();
    });

    wasm_bindgen_futures::JsFuture::from(promise).await.expect("timeout failed");
}





struct MovePacker{

}
impl MovePacker{
    fn tick(&mut self,a:[f64;2]){

    }
    fn wrap(&mut self)->Vec<u8>{
        unimplemented!();
    }
    
}
struct GameState{

}
impl GameState{
    fn tick(&mut self,a:GameDelta){

    }
}

struct GameDelta{

}

struct MoveUnpacker{

}
impl MoveUnpacker{
    fn tick(&mut self)->(bool,GameDelta){
        unimplemented!();
    }
}



use ws_stream_wasm::*;


pub struct MyWebsocket{
    socket:ws_stream_wasm::WsStream
}

impl MyWebsocket{

    pub async fn new(addr:&str)->MyWebsocket{
        let socket_create = WsMeta::connect( addr, None );
        let (_,socket)=socket_create.await.unwrap();
        MyWebsocket{
            socket
        }
    }

    pub async fn send<T>(&mut self,a:T)->Result<(),std::io::Error>{

        use futures::SinkExt;
        unimplemented!();
    }

    pub async fn recv<T>(&mut self)->Result<T,std::io::Error>{

        use futures::StreamExt;
        unimplemented!();
    }

}
pub async fn run_game() {

    let (mut socket,mut socket2)=futures::join!(
        MyWebsocket::new( "ws://127.0.0.1:3012"),
        MyWebsocket::new( "ws://127.0.0.1:3012")
    );
    
    let mut engine=Engine::new("canvas",10).unwrap();
    let mut move_acc = MovePacker{};
    let mut gamestate = GameState{};

    loop {
        socket.send(move_acc.wrap()).await;
        let mut unpacker:MoveUnpacker = socket.recv().await.unwrap();

        for _ in 0..60 {

            for event in engine.next().await.unwrap(){
                match event{
                    &Event::MouseDown(mouse_pos)=>{
                        move_acc.tick(mouse_pos);
                    }
                }
            }
            
            let (send_back_game,game_delta) = unpacker.tick();

            if send_back_game{
                socket2.send(&gamestate).await.unwrap();
            }

            gamestate.tick(game_delta);

            //draw game state
        }
    }
}




pub enum Event{
    MouseDown([f64;2])
}

pub struct GameEngine{
    events:Vec<Event>,
    perf:web_sys::Performance,
    last:f64
}





impl GameEngine{
    pub fn new(a:usize)->Self{
        let window = web_sys::window().expect("should have a window in this context");
        let perf = window
            .performance()
            .expect("performance should be available");

        let last=perf.now();

        GameEngine{
            events:Vec::new(),
            perf,
            last
        }
    }

    async fn next_tick<'a>(&'a mut self)->std::slice::Iter<'a,Event>{

        let diff=self.perf.now()-self.last;

        // First up we use `Closure::wrap` to wrap up a Rust closure and create
        // a JS closure.
        let cb = Closure::wrap(Box::new(|| {
            //log("interval elapsed!");
        }) as Box<dyn FnMut()>);


        web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(&cb.as_ref().unchecked_ref(),500).unwrap();
        
        unimplemented!();
    }

}

