use gloo::console::log;
use gloo::timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod circle_program;
pub mod dots;

pub mod utils {
    //!
    //! Helper functions to access elements
    //!
    use super::*;
    pub fn get_by_id_canvas(id: &str) -> web_sys::HtmlCanvasElement {
        gloo::utils::document()
            .get_element_by_id(id)
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw()
    }
    pub fn get_by_id_elem(id: &str) -> web_sys::HtmlElement {
        gloo::utils::document()
            .get_element_by_id(id)
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw()
    }

    pub fn get_context_webgl2_offscreen(
        canvas: &web_sys::OffscreenCanvas,
    ) -> web_sys::WebGl2RenderingContext {
        canvas
            .get_context("webgl2")
            .unwrap_throw()
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw()
    }
}

#[wasm_bindgen]
extern "C" {
    #[no_mangle]
    #[used]
    static performance: web_sys::Performance;
}

struct Timer {
    last: f64,
    frame_rate: usize,
}
impl Timer {
    fn new(frame_rate: usize) -> Timer {
        let frame_rate = ((1.0 / frame_rate as f64) * 1000.0).round() as usize;

        assert!(frame_rate > 0);
        //let window = gloo::utils::window();
        //let performance = window.performance().unwrap_throw();

        Timer {
            last: performance.now(),
            frame_rate,
        }
    }

    async fn next(&mut self) {
        //let window = gloo::utils::window();
        //let performance = window.performance().unwrap_throw();

        let tt = performance.now();
        let diff = performance.now() - self.last;

        if self.frame_rate as f64 - diff > 0.0 {
            let d = (self.frame_rate as f64 - diff) as usize;
            TimeoutFuture::new(d.try_into().unwrap_throw()).await;
        }

        self.last = tt;
    }
}

pub use main::EngineMain;
use std::marker::PhantomData;
mod main {
    use super::*;
    ///
    /// The component of the engine that runs on the main thread.
    ///
    pub struct EngineMain<T> {
        worker: std::rc::Rc<std::cell::RefCell<web_sys::Worker>>,
        shutdown_fr: futures::channel::oneshot::Receiver<()>,
        _handle: gloo::events::EventListener,
        _p: PhantomData<T>,
    }

    impl<T: 'static + Serialize> EngineMain<T> {
        ///
        /// Create the engine. Blocks until the worker thread reports that
        /// it is ready to receive the offscreen canvas.
        ///
        pub async fn new(canvas: web_sys::OffscreenCanvas) -> Self {
            let mut options = web_sys::WorkerOptions::new();
            options.type_(web_sys::WorkerType::Module);
            let worker = Rc::new(RefCell::new(
                web_sys::Worker::new_with_options("./worker.js", &options).unwrap(),
            ));

            let (shutdown_fs, shutdown_fr) = futures::channel::oneshot::channel();
            let mut shutdown_fs = Some(shutdown_fs);

            let (fs, fr) = futures::channel::oneshot::channel();
            let mut fs = Some(fs);

            let _handle =
                gloo::events::EventListener::new(&worker.borrow(), "message", move |event| {
                    let event = event.dyn_ref::<web_sys::MessageEvent>().unwrap_throw();
                    let data = event.data();
                    if let Some(s) = data.as_string() {
                        if s == "ready" {
                            if let Some(f) = fs.take() {
                                f.send(()).unwrap_throw();
                            }
                        } else if s == "close" {
                            if let Some(f) = shutdown_fs.take() {
                                f.send(()).unwrap_throw();
                            }
                        }
                    }
                });

            let _ = fr.await.unwrap_throw();

            let arr = js_sys::Array::new_with_length(1);
            arr.set(0, canvas.clone().into());
            worker
                .borrow()
                .post_message_with_transfer(&canvas, &arr)
                .unwrap_throw();

            EngineMain {
                worker,
                shutdown_fr,
                _handle,
                _p: PhantomData,
            }
        }

        ///
        /// Block until the worker thread returns.
        ///
        pub async fn join(self) {
            let _ = self.shutdown_fr.await.unwrap_throw();
        }

        ///
        /// Register a new event that will be packaged and sent to the worker thread.
        ///
        pub fn register_event(
            &mut self,
            elem: &web_sys::HtmlElement,
            event_type: &'static str,
            mut func: impl FnMut(EventData) -> T + 'static,
        ) -> gloo::events::EventListener {
            let w = self.worker.clone();

            let e = elem.clone();
            gloo::events::EventListener::new(&elem, event_type, move |event| {
                let e = EventData {
                    elem: &e,
                    event,
                    event_type,
                };

                let val = func(e);
                let a = JsValue::from_serde(&val).unwrap_throw();
                w.borrow().post_message(&a).unwrap_throw();
            })
        }
    }
}

///
/// Data that can be accessed when handling events in the main thread to help
/// construct the data to be passed to the worker thread.
///
pub struct EventData<'a> {
    pub elem: &'a web_sys::HtmlElement,
    pub event: &'a web_sys::Event,
    pub event_type: &'static str,
}

pub use worker::EngineWorker;
mod worker {
    use super::*;
    ///
    /// The component of the engine that runs on the worker thread spawn inside of worker.js.
    ///
    pub struct EngineWorker<T> {
        _handle: gloo::events::EventListener,
        queue: Rc<RefCell<Vec<T>>>,
        buffer: Vec<T>,
        timer: crate::Timer,
        canvas: Rc<RefCell<Option<web_sys::OffscreenCanvas>>>,
    }

    impl<T> Drop for EngineWorker<T> {
        fn drop(&mut self) {
            let scope: web_sys::DedicatedWorkerGlobalScope =
                js_sys::global().dyn_into().unwrap_throw();

            scope
                .post_message(&JsValue::from_str("close"))
                .unwrap_throw();
        }
    }
    impl<T: 'static + for<'a> Deserialize<'a>> EngineWorker<T> {
        ///
        /// Get the offscreen canvas.
        ///
        pub fn canvas(&self) -> web_sys::OffscreenCanvas {
            self.canvas.borrow().as_ref().unwrap_throw().clone()
        }

        ///
        /// Create the worker component of the engine.
        /// Specify the frame rate.
        /// Blocks until it receives the offscreen canvas from the main thread.
        ///
        pub async fn new(time: usize) -> EngineWorker<T> {
            let scope: web_sys::DedicatedWorkerGlobalScope =
                js_sys::global().dyn_into().unwrap_throw();

            let queue: Rc<RefCell<Vec<T>>> = std::rc::Rc::new(std::cell::RefCell::new(vec![]));

            let ca: Rc<RefCell<Option<web_sys::OffscreenCanvas>>> =
                std::rc::Rc::new(std::cell::RefCell::new(None));

            let (fs, fr) = futures::channel::oneshot::channel();
            let mut fs = Some(fs);

            let caa = ca.clone();
            let q = queue.clone();

            let _handle = gloo::events::EventListener::new(&scope, "message", move |event| {
                let event = event.dyn_ref::<web_sys::MessageEvent>().unwrap_throw();
                let data = event.data();

                if data.is_instance_of::<web_sys::OffscreenCanvas>() {
                    log!("worker:received offscreen canvas");
                    if let Some(fs) = fs.take() {
                        fs.send(()).unwrap_throw();
                    }

                    let data = data.dyn_into().unwrap_throw();
                    *caa.borrow_mut() = Some(data);
                } else {
                    //let data = data.dyn_ref::<js_sys::JsString>().unwrap_throw();
                    let e = data.into_serde().unwrap_throw();

                    q.borrow_mut().push(e);
                }
            });

            scope
                .post_message(&JsValue::from_str("ready"))
                .unwrap_throw();
            log!("worker:sent ready");

            fr.await.unwrap_throw();

            log!("worker:ready to continue");
            EngineWorker {
                _handle,
                queue,
                buffer: vec![],
                timer: crate::Timer::new(time),
                canvas: ca,
            }
        }

        ///
        /// Blocks until the next frame. Returns all events that
        /// transpired since the previous call to next.
        ///
        pub async fn next(&mut self) -> &[T] {
            self.timer.next().await;
            self.buffer.clear();
            self.buffer.append(&mut self.queue.borrow_mut());
            &self.buffer
        }
    }
}
