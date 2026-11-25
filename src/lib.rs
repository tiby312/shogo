#![allow(non_upper_case_globals)]

use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::mpsc::UnboundedSender;
use gloo::render::AnimationFrame;
use gloo::timers::future::TimeoutFuture;

use std::borrow::Cow;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Event;
use web_sys::EventTarget;

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

pub struct Data {
    timestamp: f64,
    last_timestamp: f64,
}
impl Data {
    pub fn delta(&self) -> f64 {
        self.timestamp - self.last_timestamp
    }
}
pub struct RequestAnimationFrameMan {
    tx_frame: UnboundedSender<f64>,
    anim_frame: Option<AnimationFrame>,
    rx_frame: UnboundedReceiver<f64>,
    last_timestamp: f64,
}

impl RequestAnimationFrameMan {
    pub fn new() -> Self {
        let (tx_frame, rx_frame) = futures::channel::mpsc::unbounded();

        // A shared reference to hold the current AnimationFrame handle
        let anim_frame = None;

        Self {
            tx_frame,
            anim_frame,
            rx_frame,
            last_timestamp: 0.0,
        }
    }
    pub async fn next(&mut self) -> Data {
        let timestamp = self.rx_frame.recv().await.unwrap_throw();
        let d = Data {
            timestamp,
            last_timestamp: self.last_timestamp,
        };
        self.last_timestamp = timestamp;
        d
    }
    pub fn request_animation_frame(&mut self) {
        let tx_frame = self.tx_frame.clone();
        self.anim_frame = Some(gloo::render::request_animation_frame(move |time: f64| {
            tx_frame.unbounded_send(time).unwrap_throw();
        }));
    }
}

pub fn reg<S: Into<Cow<'static, str>>, T: 'static>(
    tx: &UnboundedSender<T>,
    target: &EventTarget,
    event: S,
    mut handler: impl FnMut(&Event) -> Option<T> + 'static,
) -> gloo::events::EventListener {
    let tx = tx.clone();
    gloo::events::EventListener::new(target, event, move |e| {
        let j = handler(e);
        if let Some(j) = j {
            tx.unbounded_send(j).unwrap_throw();
        }
    })
}

pub struct Timer {
    last: f64,
    frame_rate: usize,
}
impl Timer {
    pub fn new(frame_rate: usize) -> Timer {
        let frame_rate = ((1.0 / frame_rate as f64) * 1000.0).round() as usize;

        assert!(frame_rate > 0);
        let window = gloo::utils::window();

        let performance = window.performance().unwrap_throw();

        Timer {
            last: performance.now(),
            frame_rate,
        }
    }

    pub async fn next(&mut self) -> f64 {
        let window = gloo::utils::window();
        let performance = window.performance().unwrap_throw();

        let tt = performance.now();
        let diff = performance.now() - self.last;

        if self.frame_rate as f64 - diff > 0.0 {
            let d = (self.frame_rate as f64 - diff) as usize;
            TimeoutFuture::new(d.try_into().unwrap_throw()).await;
        }

        self.last = tt;
        diff
    }
}
