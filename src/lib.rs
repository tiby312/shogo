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
    use super::*;
    pub fn get_by_id_canvas(id: &str) -> web_sys::HtmlCanvasElement {
        gloo::utils::document()
            .get_element_by_id(id)
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw()
    }
    pub fn get_context_2d(
        canvas: &web_sys::HtmlCanvasElement,
    ) -> web_sys::CanvasRenderingContext2d {
        canvas
            .get_context("2d")
            .unwrap_throw()
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw()
    }

    pub fn get_context_webgl2(
        canvas: &web_sys::HtmlCanvasElement,
    ) -> web_sys::WebGl2RenderingContext {
        canvas
            .get_context("webgl2")
            .unwrap_throw()
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

    pub fn get_by_id_elem(id: &str) -> web_sys::HtmlElement {
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

#[wasm_bindgen]
extern "C" {
    #[no_mangle]
    #[used]
    static performance: web_sys::Performance;
}

pub struct Timer {
    last: f64,
    frame_rate: usize,
}
impl Timer {
    pub fn new(frame_rate: usize) -> Timer {
        let frame_rate = ((1.0 / frame_rate as f64) * 1000.0).round() as usize;

        assert!(frame_rate > 0);
        //let window = gloo::utils::window();
        //let performance = window.performance().unwrap_throw();

        Timer {
            last: performance.now(),
            frame_rate,
        }
    }

    pub async fn next(&mut self) {
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

use std::marker::PhantomData;
pub mod main {
    use super::*;
    pub struct WorkerInterface<T> {
        pub worker: std::rc::Rc<std::cell::RefCell<web_sys::Worker>>,
        shutdown_fr: futures::channel::oneshot::Receiver<()>,
        _handle: gloo::events::EventListener,
        _p: PhantomData<T>,
    }

    impl<T: 'static + Serialize> WorkerInterface<T> {
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

            WorkerInterface {
                worker,
                shutdown_fr,
                _handle,
                _p: PhantomData,
            }
        }

        pub async fn join(self) {
            let _ = self.shutdown_fr.await.unwrap_throw();
        }

        pub fn register_event(
            &mut self,
            elem: &web_sys::HtmlElement,
            event_type: &'static str,
            mut func: impl FnMut(EventInformation) -> T + 'static,
        ) -> gloo::events::EventListener {
            let w = self.worker.clone();

            let e = elem.clone();
            gloo::events::EventListener::new(&elem, event_type, move |event| {
                let e=EventInformation{
                    elem:&e,
                    event,
                    event_type
                };

                let val = func(e);


                let a = JsValue::from_serde(&val).unwrap_throw();

                /*
                let k = &e.into_js();

                let arr = js_sys::Array::new_with_length(1);
                arr.set(0, k.into());
                w.borrow()
                    .post_message_with_transfer(k, &arr)
                    .unwrap_throw();
                */
                w.borrow().post_message(&a).unwrap_throw();
            })
        }
    }
}

pub struct EventInformation<'a>{
    pub elem:&'a web_sys::HtmlElement,
    pub event:&'a web_sys::Event,
    pub event_type:&'static str,
}

pub mod worker {
    use super::*;
    pub struct WorkerHandler<T> {
        _handle: gloo::events::EventListener,
        queue: Rc<RefCell<Vec<T>>>,
        buffer: Vec<T>,
        timer: crate::Timer,
        canvas: Rc<RefCell<Option<web_sys::OffscreenCanvas>>>,
    }

    impl<T> Drop for WorkerHandler<T> {
        fn drop(&mut self) {
            let scope: web_sys::DedicatedWorkerGlobalScope =
                js_sys::global().dyn_into().unwrap_throw();

            scope
                .post_message(&JsValue::from_str("close"))
                .unwrap_throw();
        }
    }
    impl<T: 'static + for<'a> Deserialize<'a>> WorkerHandler<T> {
        pub fn canvas(&self) -> web_sys::OffscreenCanvas {
            self.canvas.borrow().as_ref().unwrap_throw().clone()
        }

        pub async fn new(time: usize) -> WorkerHandler<T> {
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
            WorkerHandler {
                _handle,
                queue,
                buffer: vec![],
                timer: crate::Timer::new(time),
                canvas: ca,
            }
        }
        pub async fn next(&mut self) -> &[T] {
            self.timer.next().await;
            self.buffer.clear();
            self.buffer.append(&mut self.queue.borrow_mut());
            &self.buffer
        }
    }
}
