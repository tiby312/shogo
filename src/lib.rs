use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone, Debug)]
pub enum Event {
    MouseDown(web_sys::HtmlElement, web_sys::MouseEvent),
    MouseMove(web_sys::HtmlElement, web_sys::MouseEvent),
}

pub struct Engine {
    events: Rc<RefCell<Vec<Event>>>,
    buffer: Vec<Event>,
    last: f64,
    frame_rate: usize,
}

pub fn engine(frame_rate: usize) -> Engine {
    Engine::new(frame_rate)
}

impl Drop for Engine{
    fn drop(&mut self){
        self.clear_all_callbacks();
    }
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
            buffer: Vec::new(),
        }
    }

    pub fn add_on_mouse_move<K: AsRef<web_sys::HtmlElement>>(&mut self, elem2: K) {
        let elem2 = elem2.as_ref();

        let ee = self.events.clone();

        let elem = elem2.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            ee.borrow_mut().push(Event::MouseMove(elem.clone(), e));
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);

        elem2.set_onmousemove(Some(cb.as_ref().unchecked_ref()));
        //TODO dont leak.
        cb.forget();
    }
    pub fn add_on_click<K: AsRef<web_sys::HtmlElement>>(&mut self, elem2: K) {
        let elem2 = elem2.as_ref();

        let ee = self.events.clone();

        let elem = elem2.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            ee.borrow_mut().push(Event::MouseDown(elem.clone(), e));
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);

        elem2.set_onclick(Some(cb.as_ref().unchecked_ref()));
        //TODO dont leak.
        cb.forget();
    }

    pub fn clear_all_callbacks(&mut self){
        //TODO
    }

    pub async fn next<'a>(&'a mut self) -> Delta<'a> {
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

        self.buffer.clear();
        let ee = &mut self.events.borrow_mut();
        self.buffer.append(ee);

        Delta{
            events:self.buffer.iter().cloned()
        }
        

    }
}

#[non_exhaustive]
pub struct Delta<'a>{
    pub events:std::iter::Cloned<std::slice::Iter<'a, Event>>
}


async fn delay(a: usize) {
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
