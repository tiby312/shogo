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

        let (tl,tr)={
            //let canvas:web_sys::Element=canvas.dyn_into().unwrap();
            
            let bb=canvas.get_bounding_client_rect();
            let tl=bb.x();
            let tr=bb.y();
            (tl,tr)
        };

        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            ee.borrow_mut()
                .push(Event::MouseDown([e.client_x() as f64-tl, e.client_y() as f64-tr]));
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

pub struct Renderer{
    max_delay:usize
}

impl Renderer{
    pub fn new(max_delay:usize)->Renderer{
        console_log!("rendered max delay={:?}",max_delay);
        Renderer{
            max_delay
        }
    }


    pub async fn render<K>(&mut self,a:impl FnOnce()->K)->Result<K,GameError>{
        
        let mut j=None;
        self.render_simple(||{
            j=Some(a())
        }).await.unwrap();
        Ok(j.take().unwrap())
    }


    async fn render_simple(&mut self,a:impl FnOnce())->Result<(),GameError>{
        unsafe{
            let j=Box::new(a) as Box<dyn FnOnce()>;
            let j=std::mem::transmute::<Box<dyn FnOnce()>,Box< (dyn FnOnce()+'static)>>(j);
            self.render_static(j).await
        }
    }

    async fn render_static<K:'static+std::fmt::Debug>(&mut self,a:impl FnOnce()->K+'static)->Result<K,GameError>{

        let window = web_sys::window().unwrap();
        
        let (sender,receiver)=futures::channel::oneshot::channel();

        
        let cb = Closure::once(Box::new(move || {
            let j=(a)();
            sender.send(j).unwrap();
        }) as Box<dyn FnOnce()>);

        let id=window.request_animation_frame(&cb.as_ref().unchecked_ref()).unwrap();

        let de=self.max_delay;
        futures::select!{
            a = receiver.fuse() => Ok(a.unwrap()),
            _ = delay(de).fuse() => {
                window.cancel_animation_frame(id).unwrap();
                Err(GameError::SocketErr)
            }
        }

        
        
    }
}


use futures::FutureExt;
