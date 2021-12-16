use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{HtmlElement, MouseEvent};

#[derive(Clone, Debug)]
pub enum Event {
    MouseDown(HtmlElement, MouseEvent),
    MouseMove(HtmlElement, MouseEvent),
}

pub struct Engine {
    events: Rc<RefCell<Vec<Event>>>,
    buffer: Vec<Event>,
    last: f64,
    frame_rate: usize,
    callbacks: Vec<CallbackType>,
}

enum CallbackType {
    MouseMove {
        element: HtmlElement,
        #[allow(dead_code)]
        callback: Closure<dyn FnMut(MouseEvent)>,
    },
    MouseClick {
        element: HtmlElement,
        #[allow(dead_code)]
        callback: Closure<dyn FnMut(MouseEvent)>,
    },
}

pub fn engine(frame_rate: usize) -> Engine {
    Engine::new(frame_rate)
}

impl Drop for Engine {
    fn drop(&mut self) {
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
            callbacks: Vec::new(),
        }
    }

    pub fn unset_onmousemove<K: AsRef<HtmlElement>>(&mut self, elem: K) {
        let elem = elem.as_ref().clone();

        let k = (0..).zip(self.callbacks.iter()).find_map(|(i, f)| {
            if let CallbackType::MouseMove { element, .. } = f {
                (elem == *element).then(|| i)
            } else {
                None
            }
        });

        if let Some(i) = k {
            elem.set_onmousemove(None);
            self.callbacks.remove(i);
        }
    }

    pub fn unset_onclick<K: AsRef<HtmlElement>>(&mut self, elem: K) {
        let elem = elem.as_ref().clone();

        let k = (0..).zip(self.callbacks.iter()).find_map(|(i, f)| {
            if let CallbackType::MouseClick { element, .. } = f {
                (elem == *element).then(|| i)
            } else {
                None
            }
        });

        if let Some(i) = k {
            elem.set_onclick(None);
            self.callbacks.remove(i);
        }
    }

    pub fn set_onmousemove<K: AsRef<HtmlElement>>(&mut self, elem: K) {
        let elem = elem.as_ref().clone();

        let k = (0..).zip(self.callbacks.iter()).find_map(|(i, f)| {
            if let CallbackType::MouseMove { element, .. } = f {
                (elem == *element).then(|| i)
            } else {
                None
            }
        });

        let callback = {
            let ee = self.events.clone();
            let aa = elem.clone();
            Closure::wrap(Box::new(move |e: MouseEvent| {
                ee.borrow_mut().push(Event::MouseMove(aa.clone(), e));
            }) as Box<dyn FnMut(MouseEvent)>)
        };

        elem.set_onmousemove(Some(callback.as_ref().unchecked_ref()));

        //drop the callback only after it is deregistered.
        if let Some(i) = k {
            self.callbacks.remove(i);
        }

        self.callbacks.push(CallbackType::MouseMove {
            element: elem.clone(),
            callback,
        });
    }

    pub fn set_onclick<K: AsRef<HtmlElement>>(&mut self, elem: K) {
        let elem = elem.as_ref().clone();

        let k = (0..).zip(self.callbacks.iter()).find_map(|(i, f)| {
            if let CallbackType::MouseClick { element, .. } = f {
                (elem == *element).then(|| i)
            } else {
                None
            }
        });

        let callback = {
            let ee = self.events.clone();
            let aa = elem.clone();
            Closure::wrap(Box::new(move |e: MouseEvent| {
                ee.borrow_mut().push(Event::MouseDown(aa.clone(), e));
            }) as Box<dyn FnMut(MouseEvent)>)
        };

        elem.set_onclick(Some(callback.as_ref().unchecked_ref()));

        //drop the callback only after it is deregistered.
        if let Some(i) = k {
            self.callbacks.remove(i);
        }

        self.callbacks.push(CallbackType::MouseClick {
            element: elem.clone(),
            callback,
        });
    }

    pub fn clear_all_callbacks(&mut self) {
        for elem in self.callbacks.iter_mut() {
            match elem {
                CallbackType::MouseMove { element, .. } => {
                    element.set_onmousemove(None);
                }
                CallbackType::MouseClick { element, .. } => {
                    element.set_onclick(None);
                }
            }
        }

        //Actually destroy the callback closures.
        self.callbacks.clear();
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

        Delta {
            events: self.buffer.iter().cloned(),
        }
    }
}

#[non_exhaustive]
pub struct Delta<'a> {
    pub events: std::iter::Cloned<std::slice::Iter<'a, Event>>,
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
