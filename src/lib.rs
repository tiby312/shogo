use gloo;
use gloo::events::EventListener;
use gloo::timers::future::TimeoutFuture;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub mod utils {
    use super::*;
    pub fn get_canvas_by_id(id: &str) -> web_sys::HtmlCanvasElement {
        gloo::utils::document()
            .get_element_by_id(id)
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw()
    }
    pub fn get_context(
        canvas: &web_sys::HtmlCanvasElement,
        ctx: &str,
    ) -> web_sys::CanvasRenderingContext2d {
        canvas
            .get_context(ctx)
            .unwrap_throw()
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw()
    }

    pub fn get_element_by_id(id: &str) -> web_sys::HtmlElement {
        gloo::utils::document()
            .get_element_by_id(id)
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw()
    }
}

pub fn engine(frame_rate: usize) -> Engine {
    Engine::new(frame_rate)
}

use std::cell::RefCell;
use std::rc::Rc;

struct Timer {
    last: f64,
    frame_rate: usize,
}
impl Timer {
    fn new(frame_rate: usize) -> Timer {
        let frame_rate = ((1.0 / frame_rate as f64) * 1000.0).round() as usize;

        assert!(frame_rate > 0);
        let window = gloo::utils::window();
        let performance = window.performance().unwrap_throw();

        Timer {
            last: performance.now(),
            frame_rate,
        }
    }

    async fn next(&mut self) {
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

pub struct Engine {
    timer: Timer,
    events: Rc<RefCell<Vec<EventElem>>>,
    buffer: Vec<EventElem>,
}

#[non_exhaustive]
pub struct DeltaRes<'a> {
    pub events: std::iter::Cloned<std::slice::Iter<'a, EventElem>>,
}

impl Engine {
    pub fn new(frame_rate: usize) -> Engine {
        let events = Rc::new(RefCell::new(Vec::new()));

        Engine {
            timer: Timer::new(frame_rate),
            events,
            buffer: Vec::new(),
        }
    }

    pub async fn next<'a>(&'a mut self) -> DeltaRes<'a> {
        self.timer.next().await;
        {
            self.buffer.clear();
            let ee = &mut self.events.borrow_mut();
            self.buffer.append(ee);
            assert!(ee.is_empty());
        }

        DeltaRes {
            events: self.buffer.iter().cloned(),
        }
    }

    #[must_use]
    pub fn add_click(
        &mut self,
        elem: impl AsRef<web_sys::HtmlElement>,
    ) -> gloo::events::EventListener {
        let sender = self.events.clone();
        let elem = elem.as_ref().clone();
        let elem2 = elem.clone();
        EventListener::new(&elem, "click", move |event| {
            let event = event
                .dyn_ref::<web_sys::MouseEvent>()
                .unwrap_throw()
                .clone();
            let g = EventElem {
                element: elem2.clone(),
                event: Event::MouseClick(event),
            };

            sender.borrow_mut().push(g);
        })
    }

    #[must_use]
    pub fn add_mousemove(
        &mut self,
        elem: impl AsRef<web_sys::HtmlElement>,
    ) -> gloo::events::EventListener {
        let sender = self.events.clone();
        let elem = elem.as_ref().clone();
        let elem2 = elem.clone();
        EventListener::new(&elem, "mousemove", move |event| {
            let event = event
                .dyn_ref::<web_sys::MouseEvent>()
                .unwrap_throw()
                .clone();
            let g = EventElem {
                element: elem2.clone().dyn_into().unwrap_throw(),
                event: Event::MouseMove(event),
            };
            sender.borrow_mut().push(g);
        })
    }
}

#[derive(Debug, Clone)]
pub struct EventElem {
    pub element: web_sys::HtmlElement,
    pub event: Event,
}

#[derive(Debug, Clone)]
pub enum Event {
    MouseClick(web_sys::MouseEvent),
    MouseMove(web_sys::MouseEvent),
}
