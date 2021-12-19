use gloo::events::EventListener;
use gloo::timers::future::TimeoutFuture;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
pub mod points;

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


    pub mod render {
        //! Similar to [`gloo::render::request_animation_frame`] except lifetimed.
        //! 

        use std::cell::RefCell;
        use std::fmt;
        use std::rc::Rc;
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        /// Handle for [`request_animation_frame`].
        #[derive(Debug)]
        pub struct AnimationFrame<'a> {
            render_id: i32,
            _closure: Closure<dyn Fn(JsValue)>,
            callback_wrapper: Rc<RefCell<Option<CallbackWrapper>>>,
            _p: std::marker::PhantomData<&'a i32>,
        }

        struct CallbackWrapper(Box<dyn FnOnce(f64) + 'static>);
        impl fmt::Debug for CallbackWrapper {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("CallbackWrapper")
            }
        }

        impl Drop for AnimationFrame<'_> {
            fn drop(&mut self) {
                if self.callback_wrapper.borrow_mut().is_some() {
                    web_sys::window()
                        .unwrap_throw()
                        .cancel_animation_frame(self.render_id)
                        .unwrap_throw()
                }
            }
        }

        /// Calls browser's `requestAnimationFrame`. It is cancelled when the handler is dropped.
        ///
        /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame)
        pub fn request_animation_frame<'a, F>(callback_once: F) -> AnimationFrame<'a>
        where
            F: FnOnce(f64) + 'a,
        {
            let j: Box<dyn FnOnce(f64) + 'a> = Box::new(callback_once);
            let k: Box<dyn FnOnce(f64) + 'static> = unsafe { std::mem::transmute(j) };

            let callback_wrapper = Rc::new(RefCell::new(Some(CallbackWrapper(k))));
            let callback: Closure<dyn Fn(JsValue)> = {
                let callback_wrapper = Rc::clone(&callback_wrapper);
                Closure::wrap(Box::new(move |v: JsValue| {
                    let time: f64 = v.as_f64().unwrap_or(0.0);
                    let callback = callback_wrapper.borrow_mut().take().unwrap().0;
                    callback(time);
                }))
            };

            let render_id = web_sys::window()
                .unwrap_throw()
                .request_animation_frame(callback.as_ref().unchecked_ref())
                .unwrap_throw();

            AnimationFrame {
                render_id,
                _closure: callback,
                callback_wrapper,
                _p: std::marker::PhantomData,
            }
        }
    }
}

pub fn engine(frame_rate: usize) -> Engine {
    Engine::new(frame_rate)
}

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

    pub fn get_last_delta(&mut self)->DeltaRes<'_>{
        DeltaRes {
            events: self.buffer.iter().cloned(),
        }
    }

    pub async fn next(&mut self) -> DeltaRes<'_> {
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
