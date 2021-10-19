
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub enum Event{
    MouseDown([f64;2])
}


pub struct Engine{
    events:Rc<RefCell<Vec<Event>>>,
    buffer:Vec<Event>,
    last:f64,
    frame_rate:usize
}
impl Engine{
    pub fn new(canvas:&str,frame_rate:usize)->Engine{
        let frame_rate=((1.0 / frame_rate as f64) * 100.0).round() as usize;

        let events=Rc::new(RefCell::new(Vec::new()));

        let ee=events.clone();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();    
        let canvas: web_sys::HtmlCanvasElement = document.get_element_by_id(canvas).unwrap().dyn_into()?;
    
        let cb = Closure::wrap(Box::new(|e:web_sys::MouseEvent| {
            ee.borrow_mut().push(Event::MouseDown([e.client_x() as f64,e.client_y() as f64]));
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);

        canvas.set_onclick(Some(&cb.as_ref().unchecked_ref()));

        Engine{
            events,
            buffer:Vec::new(),
            last:0.0,
            frame_rate
        }
    }

    async fn next<'a>(&'a mut self)->&[Event]{
        
        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");


        let tt=performance.now();
        let diff=performance.now()-self.last;
      
        let render_r0fps=if self.frame_rate as f64-diff>0.0{
            delay((self.frame_rate as f64-diff) as usize ).await;
            false
        }else{
            true
        };
        
        self.last=tt;
        self.buffer.append(&mut self.events.borrow_mut());
        &self.buffer
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





/*
use ws_stream_wasm::*;

pub async fn run_game() {
    let socket_create = WsMeta::connect( "ws://127.0.0.1:3012", None );

    let socket2_create = WsMeta::connect( "ws://127.0.0.1:3012", None );

    let mut game_engine=GameEngine::new(60);

    let mut move_acc = MoveAccumulator;

    let mut gamestate = GameState;

    let (mut socket, _wsio) = socket_create.await.expect("websocket socket creation failed");
    let (mut socket2, _wsio) = socket2_create.await.expect("websocket socket creation failed");

    loop {
        socket.send(move_acc.finish()).unwrap();
        let mut server_client = socket.recv().await.unwrap();

        for _ in 0..60 {

            for event in game_engine.next_tick().await{
                match event{
                    &Event::MouseDown(mouse_pos)=>{
                        move_acc.add_move(mouse_pos);
                    }
                }
            }
            
            let (send_back_game,game_delta)=server_client.tick();

            if send_back_game{
                socket2.send(&gamestate).unwrap();
            }

            game_delta.apply(&mut gamestate);


            //draw game state
        }
    }
}
*/


/*
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

*/