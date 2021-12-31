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

    pub fn get_worker_global_context() -> web_sys::DedicatedWorkerGlobalScope {
        js_sys::global().dyn_into().unwrap_throw()
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

use futures::FutureExt;
use futures::Stream;
use futures::StreamExt;

///
/// Takes a stream, and continually returns a list of its items that have accumulated over
/// the specified period.
///
pub struct FrameTimer<T, K> {
    timer: Timer,
    buffer: Vec<T>,
    stream: K,
}
impl<T, K: Stream<Item = T> + std::marker::Unpin> FrameTimer<T, K> {
    pub fn new(frame_rate: usize, stream: K) -> Self {
        FrameTimer {
            timer: Timer::new(frame_rate),
            buffer: vec![],
            stream,
        }
    }
    pub async fn next(&mut self) -> &[T] {
        self.buffer.clear();
        loop {
            futures::select_biased!(
                _ = self.timer.next().fuse() =>{
                    break;
                },
                val = self.stream.next().fuse()=>{
                    self.buffer.push(val.unwrap_throw());
                }
            )
        }
        &self.buffer
    }
}

pub use main::EngineMain;
use std::marker::PhantomData;
mod main {
    use super::*;
    ///
    /// The component of the engine that runs on the main thread.
    ///
    pub struct EngineMain<MainToWorker, WorkerToMain> {
        worker: std::rc::Rc<std::cell::RefCell<web_sys::Worker>>,
        _handle: gloo::events::EventListener,
        _p: PhantomData<(MainToWorker, WorkerToMain)>,
    }

    impl<MainToWorker: 'static + Serialize, WorkerToMain: for<'a> Deserialize<'a> + 'static>
        EngineMain<MainToWorker, WorkerToMain>
    {
        ///
        /// Create the engine. Blocks until the worker thread reports that
        /// it is ready to receive the offscreen canvas.
        ///
        pub async fn new(
            canvas: web_sys::OffscreenCanvas,
        ) -> (
            Self,
            futures::channel::mpsc::UnboundedReceiver<WorkerToMain>,
        ) {
            let mut options = web_sys::WorkerOptions::new();
            options.type_(web_sys::WorkerType::Module);
            let worker = Rc::new(RefCell::new(
                web_sys::Worker::new_with_options("./worker.js", &options).unwrap(),
            ));

            let (fs, fr) = futures::channel::oneshot::channel();
            let mut fs = Some(fs);

            let (ks, kr) = futures::channel::mpsc::unbounded();
            let _handle =
                gloo::events::EventListener::new(&worker.borrow(), "message", move |event| {
                    //log!("waaa");
                    let event = event.dyn_ref::<web_sys::MessageEvent>().unwrap_throw();
                    let data = event.data();

                    let data: js_sys::Array = data.dyn_into().unwrap_throw();
                    let m = data.get(0);
                    let k = data.get(1);

                    if !m.is_null() {
                        if let Some(s) = m.as_string() {
                            if s == "ready" {
                                if let Some(f) = fs.take() {
                                    f.send(()).unwrap_throw();
                                }
                            }
                        }
                    } else {
                        let a = k.into_serde().unwrap_throw();
                        ks.unbounded_send(a).unwrap_throw();
                    }
                });

            let _ = fr.await.unwrap_throw();

            let arr = js_sys::Array::new_with_length(1);
            arr.set(0, canvas.clone().into());

            let data = js_sys::Array::new();
            data.set(0, canvas.into());
            data.set(1, JsValue::null());

            worker
                .borrow()
                .post_message_with_transfer(&data, &arr)
                .unwrap_throw();

            (
                EngineMain {
                    worker,
                    _handle,
                    _p: PhantomData,
                },
                kr,
            )
        }

        pub fn post_message(&mut self, val: MainToWorker) {
            let a = JsValue::from_serde(&val).unwrap_throw();

            let data = js_sys::Array::new();
            data.set(0, JsValue::null());
            data.set(1, a);

            self.worker.borrow().post_message(&data).unwrap_throw();
        }

        ///
        /// Register a new event that will be packaged and sent to the worker thread.
        ///
        pub fn register_event(
            &mut self,
            elem: &web_sys::HtmlElement,
            event_type: &'static str,
            mut func: impl FnMut(EventData) -> MainToWorker + 'static,
        ) -> gloo::events::EventListener {
            let w = self.worker.clone();

            let e = elem.clone();
            gloo::events::EventListener::new(elem, event_type, move |event| {
                let e = EventData {
                    elem: &e,
                    event,
                    event_type,
                };

                let val = func(e);
                let a = JsValue::from_serde(&val).unwrap_throw();

                let data = js_sys::Array::new();
                data.set(0, JsValue::null());
                data.set(1, a);

                w.borrow().post_message(&data).unwrap_throw();
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
    pub struct EngineWorker<MainToWorker, WorkerToMain> {
        _handle: gloo::events::EventListener,
        canvas: web_sys::OffscreenCanvas,
        _p: PhantomData<(MainToWorker, WorkerToMain)>,
    }

    impl<MainToWorker: 'static + for<'a> Deserialize<'a>, WorkerToMain: Serialize>
        EngineWorker<MainToWorker, WorkerToMain>
    {
        ///
        /// Get the offscreen canvas.
        ///
        pub fn canvas(&self) -> web_sys::OffscreenCanvas {
            self.canvas.clone()
        }

        ///
        /// Create the worker component of the engine.
        /// Specify the frame rate.
        /// Blocks until it receives the offscreen canvas from the main thread.
        ///
        pub async fn new() -> (
            EngineWorker<MainToWorker, WorkerToMain>,
            futures::channel::mpsc::UnboundedReceiver<MainToWorker>,
        ) {
            let scope = utils::get_worker_global_context();

            let (fs, fr) = futures::channel::oneshot::channel();
            let mut fs = Some(fs);

            let (bags, bagf) = futures::channel::mpsc::unbounded();

            let _handle = gloo::events::EventListener::new(&scope, "message", move |event| {
                let event = event.dyn_ref::<web_sys::MessageEvent>().unwrap_throw();
                let data = event.data();

                let data: js_sys::Array = data.dyn_into().unwrap_throw();
                let offscreen = data.get(0);
                let payload = data.get(1);

                if !offscreen.is_null() {
                    let offscreen: web_sys::OffscreenCanvas = offscreen.dyn_into().unwrap_throw();
                    if let Some(fs) = fs.take() {
                        fs.send(offscreen).unwrap_throw();
                    }
                }

                if !payload.is_null() {
                    let e = payload.into_serde().unwrap_throw();
                    bags.unbounded_send(e).unwrap_throw();
                }
            });

            let data = js_sys::Array::new();
            data.set(0, JsValue::from_str("ready"));
            data.set(1, JsValue::null());

            scope.post_message(&data).unwrap_throw();

            let canvas = fr.await.unwrap_throw();

            (
                EngineWorker {
                    _handle,
                    canvas,
                    _p: PhantomData,
                },
                bagf,
            )
        }

        pub fn post_message(&mut self, a: WorkerToMain) {
            let scope = utils::get_worker_global_context();

            let data = js_sys::Array::new();
            data.set(0, JsValue::null());
            data.set(1, JsValue::from_serde(&a).unwrap_throw());

            scope.post_message(&data).unwrap_throw();
        }
    }
}
