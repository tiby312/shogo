use gloo;
use gloo::events::EventListener;
use gloo::timers::future::TimeoutFuture;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub struct Engine {
    last: f64,
    frame_rate: usize,
}

pub fn engine(frame_rate: usize) -> Engine {
    Engine::new(frame_rate)
}

impl Engine {
    pub fn new(frame_rate: usize) -> Engine {
        let frame_rate = ((1.0 / frame_rate as f64) * 1000.0).round() as usize;

        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

        Engine {
            last: performance.now(),
            frame_rate,
        }
    }

    pub async fn next(&mut self) {
        let window = gloo::utils::window();
        let performance = window.performance().unwrap_throw();

        let tt = performance.now();
        let diff = performance.now() - self.last;

        if self.frame_rate as f64 - diff > 0.0 {
            let d = (self.frame_rate as f64 - diff) as usize;
            TimeoutFuture::new(d.try_into().unwrap_throw()).await;
        }

        self.last = tt;
    }
}

pub struct GameE {
    pub element: web_sys::HtmlElement,
    pub event: GameEvent,
}
pub enum GameEvent {
    MouseClick(web_sys::MouseEvent),
    MouseMove(web_sys::MouseEvent),
}

pub fn register_click<K: AsRef<web_sys::HtmlElement>>(
    sender: futures::channel::mpsc::UnboundedSender<GameE>,
    elem: K,
) -> gloo::events::EventListener {
    let elem = elem.as_ref().clone();
    let ssender = sender.clone();
    let elem2 = elem.clone();
    EventListener::new(&elem, "click", move |event| {
        let event = event
            .dyn_ref::<web_sys::MouseEvent>()
            .unwrap_throw()
            .clone();
        let g = GameE {
            element: elem2.clone(),
            event: GameEvent::MouseClick(event),
        };
        ssender.unbounded_send(g).unwrap_throw();
    })
}

pub fn register_mousemove<K: AsRef<web_sys::HtmlElement>>(
    sender: futures::channel::mpsc::UnboundedSender<GameE>,
    elem: K,
) -> gloo::events::EventListener {
    let elem = elem.as_ref().clone();
    let ssender = sender.clone();
    let elem2 = elem.clone();
    EventListener::new(&elem, "mousemove", move |event| {
        let event = event
            .dyn_ref::<web_sys::MouseEvent>()
            .unwrap_throw()
            .clone();
        let g = GameE {
            element: elem2.clone().dyn_into().unwrap_throw(),
            event: GameEvent::MouseMove(event),
        };
        ssender.unbounded_send(g).unwrap_throw();
    })
}
