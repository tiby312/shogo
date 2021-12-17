use gloo;
use gloo::console::log;
use gloo::events::EventListener;
use gloo::timers::future::TimeoutFuture;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;



pub mod utils{
    use super::*;
    pub fn get_canvas_by_id(id:&str)->web_sys::HtmlCanvasElement{
        gloo::utils::document()
        .get_element_by_id(id)
        .unwrap_throw()
        .dyn_into().unwrap_throw()

    }
    pub fn get_context(canvas:&web_sys::HtmlCanvasElement,ctx:&str)->web_sys::CanvasRenderingContext2d{
        canvas.get_context(ctx).unwrap_throw().unwrap_throw().dyn_into().unwrap_throw()
    }

    pub fn get_element_by_id(id:&str)->web_sys::HtmlElement{
        gloo::utils::document()
        .get_element_by_id(id)
        .unwrap_throw()
        .dyn_into().unwrap_throw()

    }
}
pub struct Engine {
    last: f64,
    frame_rate: usize,
}



pub fn event_engine() -> EventEngine {
    EventEngine::new()
}
pub fn frame_engine(frame_rate: usize) -> Engine {
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

#[derive(Debug)]
pub struct GameE {
    pub element: web_sys::HtmlElement,
    pub event: GameEvent,
}

#[derive(Debug)]
pub enum GameEvent {
    MouseClick(web_sys::MouseEvent),
    MouseMove(web_sys::MouseEvent),
}


pub struct EventEngine {
    sender: futures::channel::mpsc::Sender<GameE>,
    receiver: futures::channel::mpsc::Receiver<GameE>,
}

impl EventEngine {
    pub fn new() -> EventEngine {
        let (sender, receiver) = futures::channel::mpsc::channel(20);
        EventEngine { sender, receiver }
    }

    pub async fn next(&mut self) -> GameE {
        use futures::future::FutureExt;
        use futures::stream::StreamExt;
        self.receiver.next().map(|x| x.unwrap_throw()).await
    }


    #[must_use]
    pub fn register_click<K: AsRef<web_sys::HtmlElement>>(
        &mut self,
        elem: K,
    ) -> gloo::events::EventListener {
        let mut sender = self.sender.clone();
        let elem = elem.as_ref().clone();
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
            if let Err(_) = sender.try_send(g) {
                log!("failed to queue event!")
            }
        })
    }


    #[must_use]
    pub fn register_mousemove<K: AsRef<web_sys::HtmlElement>>(
        &mut self,
        elem: K,
    ) -> gloo::events::EventListener {
        let mut sender = self.sender.clone();
        let elem = elem.as_ref().clone();
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
            if let Err(_) = sender.try_send(g) {
                log!("failed to queue event!");
            }
        })
    }
}
