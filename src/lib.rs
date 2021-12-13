use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone,Debug)]
pub enum Event {
    MouseDown(web_sys::HtmlElement, web_sys::MouseEvent),
    MouseMove(web_sys::HtmlElement, web_sys::MouseEvent),
}

pub struct Engine {
    events: Rc<RefCell<Vec<Event>>>,
    last: f64,
    frame_rate: usize,
}

impl Engine {
    pub fn new(frame_rate: usize) -> Engine {
        let frame_rate = ((1.0 / frame_rate as f64) * 1000.0).round() as usize;

        let events = Rc::new(RefCell::new(Vec::new()));

        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

        Engine {
            events,
            last: performance.now(),
            frame_rate,
        }
    }

    pub fn add_on_mouse_move<K:AsRef<web_sys::HtmlElement>>(&mut self, elem2: K) {
        let elem2=elem2.as_ref();

        let ee = self.events.clone();

        let elem = elem2.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            ee.borrow_mut().push(Event::MouseMove(elem.clone(), e));
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);

        elem2.set_onmousemove(Some(cb.as_ref().unchecked_ref()));
        //TODO dont leak.
        cb.forget();
    }
    pub fn add_on_click<K:AsRef<web_sys::HtmlElement>>(&mut self, elem2: K) {
        let elem2=elem2.as_ref();

        let ee = self.events.clone();

        let elem = elem2.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            ee.borrow_mut().push(Event::MouseDown(elem.clone(), e));
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);

        elem2.set_onclick(Some(cb.as_ref().unchecked_ref()));
        //TODO dont leak.
        cb.forget();
    }

    pub async fn next(&mut self) -> Option<Vec<Event>> {
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
        
        let mut buffer=Vec::new();
        let ee = &mut self.events.borrow_mut();
        buffer.append(ee);
        assert!(ee.is_empty());
    

        Some(buffer)
    }
}

pub async fn delay(a: usize) {
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

/*
use ws_stream_wasm::*;

pub struct MyWebsocket {
    socket: ws_stream_wasm::WsStream,
}

impl MyWebsocket {
    pub async fn new(addr: &str) -> Result<MyWebsocket, GameError> {
        let socket_create = WsMeta::connect(addr, None);
        let (_, socket) = socket_create.await.map_err(|_| GameError::SocketErr)?;
        Ok(MyWebsocket { socket })
    }

    pub async fn send<T>(&mut self, a: T) -> Result<(), GameError> {
        use futures::SinkExt;
        unimplemented!();
    }

    pub async fn recv<T>(&mut self) -> Result<T, GameError> {
        use futures::StreamExt;
        unimplemented!();
    }
}

#[derive(Debug, Copy, Clone)]
pub enum GameError {
    SocketErr,
}

pub struct Renderer {
    max_delay: usize,
}

impl Renderer {
    pub fn new(max_delay: usize) -> Renderer {
        console_log!("rendered max delay={:?}", max_delay);
        Renderer { max_delay }
    }

    pub async fn render<K>(&mut self, a: impl FnOnce() -> K) -> Result<K, GameError> {
        let mut j = None;
        self.render_simple(|| j = Some(a())).await.unwrap();
        Ok(j.take().unwrap())
    }

    async fn render_simple(&mut self, a: impl FnOnce()) -> Result<(), GameError> {
        unsafe {
            let j = Box::new(a) as Box<dyn FnOnce()>;
            let j = std::mem::transmute::<Box<dyn FnOnce()>, Box<(dyn FnOnce() + 'static)>>(j);
            self.render_static(j).await
        }
    }

    async fn render_static<K: 'static + std::fmt::Debug>(
        &mut self,
        a: impl FnOnce() -> K + 'static,
    ) -> Result<K, GameError> {
        let window = web_sys::window().unwrap();

        let (sender, receiver) = futures::channel::oneshot::channel();

        let cb = Closure::once(Box::new(move || {
            let j = (a)();
            sender.send(j).unwrap();
        }) as Box<dyn FnOnce()>);

        let id = window
            .request_animation_frame(&cb.as_ref().unchecked_ref())
            .unwrap();

        let de = self.max_delay;
        futures::select! {
            a = receiver.fuse() => Ok(a.unwrap()),
            _ = delay(de).fuse() => {
                window.cancel_animation_frame(id).unwrap();
                Err(GameError::SocketErr)
            }
        }
    }
}

use futures::FutureExt;

*/
