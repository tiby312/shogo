#![allow(non_upper_case_globals)]

use gloo::timers::future::TimeoutFuture;
use main::Transferable;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use gloo::utils::format::JsValueSerdeExt;

//pub mod simple2d;

pub mod utils {
    //!
    //! Helper functions to access elements
    //!
    use super::*;

    ///
    /// Get an element with the specified id.
    ///
    pub fn get_by_id_elem(id: &str) -> web_sys::HtmlElement {
        gloo::utils::document()
            .get_element_by_id(id)
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw()
    }

    ///
    /// Get a canvas element with the specified id.
    ///
    pub fn get_by_id_canvas(id: &str) -> web_sys::HtmlCanvasElement {
        gloo::utils::document()
            .get_element_by_id(id)
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw()
    }

    ///
    /// Get a webgl2 context for an offscreen canvas element.
    ///
    pub fn get_context_webgl2_offscreen(
        canvas: &web_sys::OffscreenCanvas,
    ) -> web_sys::WebGl2RenderingContext {
        let options = web_sys::WebGlContextAttributes::new();
        options.set_antialias(true);
        options.set_alpha(true);
        canvas
            .get_context_with_context_options("webgl2", &*options)
            .unwrap_throw()
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw()
    }

    ///
    /// Get the worker global scope. Call from within a webworker.
    ///
    pub fn get_worker_global_context() -> web_sys::DedicatedWorkerGlobalScope {
        js_sys::global().dyn_into().unwrap_throw()
    }
}

// #[wasm_bindgen]
// extern "C" {
//     #[no_mangle]
//     #[used]
//     #[wasm_bindgen(thread_local)]
//     static performance: web_sys::Performance;
// }

pub struct Timer {
    last: f64,
    frame_rate: usize,
}
impl Timer {
    pub fn new(frame_rate: usize) -> Timer {
        let frame_rate = ((1.0 / frame_rate as f64) * 1000.0).round() as usize;

        assert!(frame_rate > 0);
        //let window = gloo::utils::window();
        let performance = utils::get_worker_global_context()
            .performance()
            .unwrap_throw();

        //let performance = window.performance().unwrap_throw();

        Timer {
            last: performance.now(),
            frame_rate,
        }
    }

    pub async fn next(&mut self) {
        //let window = gloo::utils::window();
        //let performance = window.performance().unwrap_throw();
        let performance = utils::get_worker_global_context()
            .performance()
            .unwrap_throw();

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

// ///
// /// Takes a stream, and continually returns a list of its items that have accumulated over
// /// the specified period.
// ///
// pub struct FrameTimer<T, K> {
//     timer: Timer,
//     buffer: Vec<T>,
//     stream: K,
// }
// impl<T, K: Stream<Item = T> + std::marker::Unpin> FrameTimer<T, K> {
//     pub fn new(frame_rate: usize, stream: K) -> Self {
//         FrameTimer {
//             timer: Timer::new(frame_rate),
//             buffer: vec![],
//             stream,
//         }
//     }
//     pub async fn next(&mut self) -> Doop<T> {
//         //self.buffer.clear();

//         loop {
//             futures::select_biased!(
//                 _ = self.timer.next().fuse() =>{
//                     break;
//                 },
//                 val = self.stream.next().fuse()=>{
//                     self.buffer.push(val.unwrap_throw());
//                 }
//             )
//         }
//         Doop{inner:&mut self.buffer}
//     }
// }
// pub struct Doop<'a,T>{
//     inner:&'a mut Vec<T>
// }
// impl<'a,T> Doop<'a,T>{
//     pub fn events(&self)->&[T]{
//         &self.inner
//     }
// }
// impl<'a,T> Drop for Doop<'a,T>{
//     fn drop(&mut self) {
//         self.inner.clear();
//     }
// }

use gloop::Listen;
//pub use main::EngineMain;
use std::marker::PhantomData;
pub mod main {
    use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};

    use super::*;

    pub struct MyListen<WM> {
        ks: UnboundedSender<WM>,
        fs: Option<futures::channel::oneshot::Sender<()>>,
    }

    impl<WM: for<'a> Deserialize<'a>> Listen for MyListen<WM> {
        fn call(&mut self, event: &web_sys::Event) {
            let event = event.dyn_ref::<web_sys::MessageEvent>().unwrap_throw();
            let data = event.data();

            let data: js_sys::Array = data.dyn_into().unwrap_throw();
            let m = data.get(0);
            let k = data.get(1);

            if !m.is_null() {
                if let Some(s) = m.as_string() {
                    if s == "ready" {
                        if let Some(f) = self.fs.take() {
                            f.send(()).unwrap_throw();
                        }
                    }
                }
            } else {
                let a = k.into_serde().unwrap_throw();
                self.ks.unbounded_send(a).unwrap_throw();
            }
        }
    }

    pub struct MainReceiver<WM> {
        _handle: gloop::EventListen<MyListen<WM>>,
        recv: futures::channel::mpsc::UnboundedReceiver<WM>,
    }
    impl<WM> MainReceiver<WM> {
        pub fn recv(&mut self) -> &mut futures::channel::mpsc::UnboundedReceiver<WM> {
            &mut self.recv
        }
    }

    pub struct MainSender<MW> {
        worker: std::rc::Rc<std::cell::RefCell<web_sys::Worker>>,
        _p: PhantomData<MW>,
    }
    impl<MW: Serialize> MainSender<MW> {
        pub fn post_message(&self, val: MW) {
            let a = JsValue::from_serde(&val).unwrap_throw();

            let data = js_sys::Array::new();
            data.set(0, JsValue::null());
            data.set(1, a);

            self.worker.borrow().post_message(&data).unwrap_throw();
        }
    }

    pub trait Transferable : Clone+Into<JsValue>+wasm_bindgen::JsCast+ std::fmt::Debug{

    }
    impl Transferable for web_sys::OffscreenCanvas{}
    impl Transferable for js_sys::ArrayBuffer{}
    
    pub async fn create_main<MW: Serialize, WM: for<'a> Deserialize<'a>,T:Transferable>(
        web_worker_url: &str,
        canvas: T,
    ) -> (MainSender<MW>, MainReceiver<WM>) {
        let options = web_sys::WorkerOptions::new();
        options.set_type(web_sys::WorkerType::Module);
        let worker = Rc::new(RefCell::new(
            web_sys::Worker::new_with_options(web_worker_url, &options).unwrap_throw(),
        ));

        let (fs, fr) = futures::channel::oneshot::channel();
        let fs = Some(fs);

        let (ks, kr) = futures::channel::mpsc::unbounded();
        let ks: UnboundedSender<WM> = ks;
        let kr: UnboundedReceiver<WM> = kr;

        let ml = MyListen { ks, fs };

        let _handle = gloop::EventListen::new(&worker.borrow(), "message", ml);

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
            MainSender {
                worker,
                _p: PhantomData,
            },
            MainReceiver { _handle, recv: kr },
        )
    }

    // ///
    // /// The component of the engine that runs on the main thread.
    // ///
    // pub struct EngineMain<MW, WM> {
    //     worker: std::rc::Rc<std::cell::RefCell<web_sys::Worker>>,
    //     _handle: gloop::EventListen<MyListen<WM>>,
    //     _p: PhantomData<(MW, WM)>,
    // }

    // impl<MW:  Serialize, WM: for<'a> Deserialize<'a>> EngineMain<MW, WM> {
    //     ///
    //     /// Create the engine. Blocks until the worker thread reports that
    //     /// it is ready to receive the offscreen canvas.
    //     ///
    //     pub async fn new(
    //         web_worker_url: &str,
    //         canvas: web_sys::OffscreenCanvas,
    //     ) -> (Self, futures::channel::mpsc::UnboundedReceiver<WM>) {
    //         let options = web_sys::WorkerOptions::new();
    //         options.set_type(web_sys::WorkerType::Module);
    //         let worker = Rc::new(RefCell::new(
    //             web_sys::Worker::new_with_options(web_worker_url, &options).unwrap_throw(),
    //         ));

    //         let (fs, fr) = futures::channel::oneshot::channel();
    //         let fs = Some(fs);

    //         let (ks, kr) = futures::channel::mpsc::unbounded();
    //         let ks:UnboundedSender<WM>=ks;
    //         let kr:UnboundedReceiver<WM>=kr;

    //         let ml=MyListen{
    //             ks,
    //             fs
    //         };

    //         let _handle = gloop::EventListen::new(&worker.borrow(),"message",ml);

    //         let _ = fr.await.unwrap_throw();

    //         let arr = js_sys::Array::new_with_length(1);
    //         arr.set(0, canvas.clone().into());

    //         let data = js_sys::Array::new();
    //         data.set(0, canvas.into());
    //         data.set(1, JsValue::null());

    //         worker
    //             .borrow()
    //             .post_message_with_transfer(&data, &arr)
    //             .unwrap_throw();

    //         (
    //             EngineMain {
    //                 worker,
    //                 _handle,
    //                 _p: PhantomData,
    //             },
    //             kr,
    //         )
    //     }

    //     pub fn post_message(&self, val: MW) {
    //         let a = JsValue::from_serde(&val).unwrap_throw();

    //         let data = js_sys::Array::new();
    //         data.set(0, JsValue::null());
    //         data.set(1, a);

    //         self.worker.borrow().post_message(&data).unwrap_throw();
    //     }

    // }
}

pub struct MyListen2<F> {
    func: F,
    e: web_sys::EventTarget,
    event_type: &'static str,
    //TODO dont use reference counting
    w: Rc<RefCell<web_sys::Worker>>,
}
impl<'a, MW: Serialize, F: FnMut(EventData) -> Option<MW>> gloop::Listen for MyListen2<F> {
    fn call(&mut self, event: &web_sys::Event) {
        let e = EventData {
            elem: &self.e,
            event,
            event_type: self.event_type,
        };

        if let Some(val) = (self.func)(e) {
            let a = JsValue::from_serde(&val).unwrap_throw();

            let data = js_sys::Array::new();
            data.set(0, JsValue::null());
            data.set(1, a);

            self.w.borrow().post_message(&data).unwrap_throw();
        }
    }
}

///
/// Data that can be accessed when handling events in the main thread to help
/// construct the data to be passed to the worker thread.
///
pub struct EventData<'a> {
    pub elem: &'a web_sys::EventTarget,
    pub event: &'a web_sys::Event,
    pub event_type: &'static str,
}

//pub use worker::EngineWorker;
pub mod worker {
    use web_sys::OffscreenCanvas;

    use crate::main::Transferable;

    use super::*;
    // ///
    // /// The component of the engine that runs on the worker thread spawn inside of worker.js.
    // ///
    // pub struct EngineWorker<MW, WM> {
    //     _handle: gloop::EventListen<MyListen3<MW>>,
    //     canvas: web_sys::OffscreenCanvas,
    //     _p: PhantomData<(MW, WM)>,
    // }

    pub struct WorkerSender<WM> {
        _p: PhantomData<WM>,
    }
    impl<WM: Serialize> WorkerSender<WM> {
        pub fn post_message(&self, a: WM) {
            let scope = utils::get_worker_global_context();

            let data = js_sys::Array::new();
            data.set(0, JsValue::null());
            data.set(1, JsValue::from_serde(&a).unwrap_throw());

            scope.post_message(&data).unwrap_throw();
        }
    }

    pub struct WorkerRecv<MW,T:Transferable> {
        _handle: gloop::EventListen<MyListen3<MW,T>>,
        //canvas: web_sys::OffscreenCanvas,
        recv: futures::channel::mpsc::UnboundedReceiver<MW>,
    }
    impl<MW,T:Transferable> WorkerRecv<MW,T> {
        pub fn recv(&mut self) -> &mut futures::channel::mpsc::UnboundedReceiver<MW> {
            &mut self.recv
        }
    }

    pub async fn create_worker<WM: Serialize, MW: for<'a> Deserialize<'a>,T:Transferable>(
    ) -> (T, WorkerSender<WM>, WorkerRecv<MW,T>) {
        let scope = utils::get_worker_global_context();

        let (fs, fr): (futures::channel::oneshot::Sender<T>, _) =
            futures::channel::oneshot::channel();
        let fs = Some(fs);

        let (bags, bagf): (futures::channel::mpsc::UnboundedSender<MW>, _) =
            futures::channel::mpsc::unbounded();

        let fff = MyListen3 { fs, bags };

        let _handle = gloop::EventListen::new(&scope, "message", fff);

        let data = js_sys::Array::new();
        data.set(0, JsValue::from_str("ready"));
        data.set(1, JsValue::null());

        scope.post_message(&data).unwrap_throw();

        let canvas = fr.await.unwrap_throw();

        (
            canvas,
            WorkerSender { _p: PhantomData },
            WorkerRecv {
                _handle,
                recv: bagf,
            },
        )
    }

    // impl<MW: for<'a> Deserialize<'a>, WM: Serialize> EngineWorker<MW, WM> {
    //     ///
    //     /// Get the offscreen canvas.
    //     ///
    //     pub fn canvas(&self) -> web_sys::OffscreenCanvas {
    //         self.canvas.clone()
    //     }

    //     ///
    //     /// Create the worker component of the engine.
    //     /// Specify the frame rate.
    //     /// Blocks until it receives the offscreen canvas from the main thread.
    //     ///
    //     pub async fn new() -> (
    //         EngineWorker<MW, WM>,
    //         futures::channel::mpsc::UnboundedReceiver<MW>,
    //     ) {
    //         let scope = utils::get_worker_global_context();

    //         let (fs, fr) :(futures::channel::oneshot::Sender<OffscreenCanvas>,_)= futures::channel::oneshot::channel();
    //         let fs = Some(fs);

    //         let (bags, bagf):(futures::channel::mpsc::UnboundedSender<MW>,_) = futures::channel::mpsc::unbounded();

    //         let fff=MyListen3{
    //             fs,
    //             bags
    //         };

    //         let _handle=gloop::EventListen::new(&scope,"message",fff);

    //         let data = js_sys::Array::new();
    //         data.set(0, JsValue::from_str("ready"));
    //         data.set(1, JsValue::null());

    //         scope.post_message(&data).unwrap_throw();

    //         let canvas = fr.await.unwrap_throw();

    //         (
    //             EngineWorker {
    //                 _handle,
    //                 canvas,
    //                 _p: PhantomData,
    //             },
    //             bagf,
    //         )
    //     }

    //     pub fn post_message(&mut self, a: WM) {
    //         let scope = utils::get_worker_global_context();

    //         let data = js_sys::Array::new();
    //         data.set(0, JsValue::null());
    //         data.set(1, JsValue::from_serde(&a).unwrap_throw());

    //         scope.post_message(&data).unwrap_throw();
    //     }
    // }
}

pub struct MyListen3<MW,T:main::Transferable> {
    fs: Option<futures::channel::oneshot::Sender<T>>,
    bags: futures::channel::mpsc::UnboundedSender<MW>,
}

impl<MW: for<'a> Deserialize<'a>,T:Transferable> gloop::Listen for MyListen3<MW,T> {
    fn call(&mut self, event: &web_sys::Event) {
        let event = event.dyn_ref::<web_sys::MessageEvent>().unwrap_throw();
        let data = event.data();

        let data: js_sys::Array = data.dyn_into().unwrap_throw();
        let offscreen = data.get(0);
        let payload = data.get(1);

        if !offscreen.is_null() {
            let offscreen: T = offscreen.dyn_into().unwrap_throw();
            if let Some(fs) = self.fs.take() {
                fs.send(offscreen).unwrap_throw();
            }
        }

        if !payload.is_null() {
            let e = payload.into_serde().unwrap_throw();
            self.bags.unbounded_send(e).unwrap_throw();
        }
    }
}
