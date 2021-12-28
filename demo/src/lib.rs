use gloo::console::log;
use wasm_bindgen::prelude::*;

use shogo::{
    dots::{CtxExt, Shapes},
    utils,
};

#[wasm_bindgen(start)]
pub async fn init_module(){
    log!("initing a module");
}


/*
///Call from worker.
pub fn register_click(foo:F,elem:&HtmlElement)
{
    
    let scope:web_sys::DedicatedWorkerGlobalScope =js_sys::global().dyn_into().unwrap_throw();
    
    let foo=Closure::once_into_js(foo);

    let k:&js_sys::Object=foo.dyn_ref().unwrap_throw();

    log!(k.to_string());
    
    let arr=js_sys::Array::new_with_length(1);
    arr.set(0,foo);

    let blob=web_sys::Blob::new_with_buffer_source_sequence(&arr).unwrap_throw();

    let s:js_sys::JsString=blob.to_string();
    log!("logged closure");
    scope.post_message(&s).unwrap_throw();


}
*/

use wasm_bindgen::JsCast;
    



#[derive(Debug, Clone)]
pub enum MEvent {
    MouseMove{elem:arrayvec::ArrayString<30>,client_x:f64,client_y:f64},
}

impl MEvent{
    fn into_js(self)->js_sys::ArrayBuffer{
        let l=std::mem::size_of::<Self>();
        let arr:&[u8]=unsafe{std::slice::from_raw_parts(&self as *const _ as *const _,l)};
        let buffer=js_sys::Uint8Array::new_with_length(l as u32);
        buffer.copy_from(arr);
        buffer.buffer()
    }
    fn from_js(ar:&js_sys::ArrayBuffer)->MEvent{
        let ar=js_sys::Uint8Array::new_with_byte_offset(ar,0);

        let mut j:MEvent=unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        let l=std::mem::size_of::<Self>();
        let arr:&mut [u8]=unsafe{std::slice::from_raw_parts_mut (&mut j as *mut _ as *mut _,l)};
        ar.copy_to(arr);
        j
    }
}


mod main{
    use super::*;
    pub struct WorkerInterface{
        pub worker:std::rc::Rc<std::cell::RefCell<web_sys::Worker>>
    }

    impl WorkerInterface{
        pub async fn new(canvas:web_sys::OffscreenCanvas)->Self{
            let mut options=web_sys::WorkerOptions::new();
            options.type_(web_sys::WorkerType::Module);
            let worker = Rc::new(RefCell::new(web_sys::Worker::new_with_options("./worker.js",&options).unwrap()));
            

            
            use futures::SinkExt;
            use futures::StreamExt;

            {
                let (fs,mut fr)=futures::channel::oneshot::channel();
                let mut fs=Some(fs);

                let _handle=gloo::events::EventListener::new(&worker.borrow(), "message", move |event| {
                    log!("main:worker is ready!");
                    if let Some(f)=fs.take(){
                        f.send(()).unwrap_throw();
                    }
                });

                let _ =fr.await.unwrap_throw();
            }
            log!("main:continuing");

            //TODO fix
            //TimeoutFuture::new(100).await;

            let arr=js_sys::Array::new_with_length(1);
            arr.set(0,canvas.clone().into());
            worker.borrow().post_message_with_transfer(&canvas,&arr).unwrap_throw();

            WorkerInterface{
                worker
            }
        }


        pub fn register_mousemove_handler(&mut self,elem:&web_sys::HtmlCanvasElement)->gloo::events::EventListener{
            let w=self.worker.clone();

            let e=elem.clone();
            gloo::events::EventListener::new(&elem, "mousemove", move |event| {
                let event = event
                .dyn_ref::<web_sys::MouseEvent>()
                .unwrap_throw()
                .clone();

                
                let e=MEvent::MouseMove{
                    elem:arrayvec::ArrayString::from(&e.id()).unwrap_throw(),
                    client_x:event.client_x() as f64,
                    client_y:event.client_y() as f64,
                };

                let k=&e.into_js();

                let arr=js_sys::Array::new_with_length(1);
                arr.set(0,k.into());
                w.borrow().post_message_with_transfer(k,&arr).unwrap_throw();
            })
        }
    }
}


use std::cell::RefCell;
use std::rc::Rc;

mod worker{
    use super::*;
    pub struct WorkerHandler{
        _handle:gloo::events::EventListener,
        queue:Rc<RefCell<Vec<MEvent>>>,
        buffer:Vec<MEvent>,
        timer:shogo::Timer,
        canvas:Rc<RefCell<Option<web_sys::OffscreenCanvas>>>
    }

    impl WorkerHandler{
        pub fn canvas(&self)->web_sys::OffscreenCanvas{
            self.canvas.borrow().as_ref().unwrap_throw().clone()
        }

        pub async fn new(time:usize)->WorkerHandler{
            let scope:web_sys::DedicatedWorkerGlobalScope =js_sys::global().dyn_into().unwrap_throw();
        

            let queue:Rc<RefCell<Vec<MEvent>>>=std::rc::Rc::new(std::cell::RefCell::new(vec![]));

            let ca:Rc<RefCell<Option<web_sys::OffscreenCanvas>>>=std::rc::Rc::new(std::cell::RefCell::new(None));

            let (fs,mut fr)=futures::channel::oneshot::channel();
            let mut fs=Some(fs);

            let caa=ca.clone();
            let q=queue.clone();
            
            
            
            let _handle=gloo::events::EventListener::new(&scope, "message", move |event| {
                let event=event.dyn_ref::<web_sys::MessageEvent>().unwrap_throw();
                log!(event);
                let data=event.data();

                
                if data.is_instance_of::<js_sys::ArrayBuffer>(){

                    let data=data.dyn_ref::<js_sys::ArrayBuffer>().unwrap_throw();

                    let e=MEvent::from_js(&data);

                    q.borrow_mut().push(e);
                }else if data.is_instance_of::<web_sys::OffscreenCanvas>(){
                    
                    if let Some(fs)=fs.take(){
                        fs.send(()).unwrap_throw();
                    }

                    log!("got offscreen canvas!");
                    let data=data.dyn_into().unwrap_throw();
                    *caa.borrow_mut()=Some(data);
                }else{
                    log!("got something unexpected!");
                }
            });
            

            log!("workering:posting ready message!");
            scope.post_message(&JsValue::from_str("ready")).unwrap_throw();


            use futures::StreamExt;
            fr.await.unwrap_throw();


            WorkerHandler{
                _handle,
                queue,
                buffer:vec![],
                timer:shogo::Timer::new(time),
                canvas:ca
            }
        }
        pub async fn next(&mut self)->&[MEvent]{
            self.timer.next().await;
            self.buffer.clear();
            self.buffer.append(&mut self.queue.borrow_mut());
            &self.buffer
        }
    }
}





#[wasm_bindgen]
pub async fn worker_entry(){
    let scope:web_sys::DedicatedWorkerGlobalScope =js_sys::global().dyn_into().unwrap_throw();


    let mut w=worker::WorkerHandler::new(30).await;

    let canvas=w.canvas();
    log!(canvas);
    loop{
        for e in w.next().await{
            match e{
                MEvent::MouseMove{elem,client_x,client_y}=>{
                    match elem.as_str(){
                        "mycanvas"=>{
                            //gloo::console::console_dbg!(elem,client_x,client_y);
                            //log!("got mouse move!")
                        },
                        _=>{}
                    }
                    
                }
            }
        }
    }

    /*    
    log!(foo);

    //log!("i'm in a worker2!");
    //log!("global=",js_sys::global());
    let scope:web_sys::DedicatedWorkerGlobalScope =js_sys::global().dyn_into().unwrap_throw();

    /*
    let arr=js_sys::Array::new_with_length(2);
    arr.set(0,JsValue::from_str("hello"));
    arr.set(1,JsValue::from(5u32));
    scope.post_message(&arr).unwrap_throw();
    */
    //let messages=std::rc::Rc::new(std::cell::RefCell::new(vec!()));
    //let m=messages.clone();
    
    use gloo::timers::future::TimeoutFuture;

    
    let _handle=gloo::events::EventListener::new(&scope, "message", move |event| {
        /*
        let event=event.dyn_ref::<web_sys::MessageEvent>().unwrap_throw();
        let data=event.data();
        let arr=data.dyn_ref::<js_sys::Array>().unwrap_throw();
        let s:js_sys::JsString=arr.get(0).dyn_into().unwrap_throw();
        let s:String=s.into();

        messages.borrow_mut().push(s);
        
        use wasm_bindgen::JsCast;
        */
        log!(event);
    });


    let mut timer=shogo::Timer::new(30);
    loop{
        timer.next().await;
        log!("next");


    }
    */

}
use gloo::timers::future::TimeoutFuture;


#[wasm_bindgen]
pub async fn main_entry() {
    log!("demo start!");
    
    use std::cell::RefCell;
    use std::rc::Rc;
    use web_sys::Worker;
    
    
    /*
    let _handle=gloo::events::EventListener::new(&worker_handle.borrow(), "message", move |event| {
        log!(event);
        /*
        let event=event.dyn_ref::<web_sys::MessageEvent>().unwrap_throw();
        
        let data=event.data();
        let arr=data.dyn_ref::<js_sys::Array>().unwrap_throw();

        let s:js_sys::JsString=arr.get(0).dyn_into().unwrap_throw();
        let s:String=s.into();

        log!(s);

        //log!("main got messaage!!!");
        
        use wasm_bindgen::JsCast;
        */


        /*
        //if even.is_instance_of::<js_sys::Function>(){
            log!("main entry:got function!");
            
            let k:js_sys::Function=event.data().dyn_into().unwrap_throw();
            k.call0(&wasm_bindgen::JsValue::null());
            log!("main entry:finished calling function");
            
        //}
        */
    });
    */

    

    let (canvas, button, shutdown_button) = (
        utils::get_by_id_canvas("mycanvas"),
        utils::get_by_id_elem("mybutton"),
        utils::get_by_id_elem("shutdownbutton"),
    );



    let mut worker=main::WorkerInterface::new(canvas.transfer_control_to_offscreen().unwrap_throw()).await;

    let _handler=worker.register_mousemove_handler(&canvas);

    
    TimeoutFuture::new(100000).await;

    /*
    let ctx = utils::get_context_webgl2(&canvas);

    let mut engine = shogo::engine(60);

    let _handle = engine.add_mousemove(&canvas);
    let _handle = engine.add_click(&button);
    let _handle = engine.add_click(&shutdown_button);

    


    let mut mouse_pos = [0.0f32; 2];

    let mut color_iter = {
        let colors = [
            [1.0, 0.0, 0.0, 0.5],
            [0.0, 1.0, 0.0, 0.5],
            [0.0, 0.0, 1.0, 0.5],
        ];
        colors.into_iter().cycle().peekable()
    };

    let (mut draw_sys, mut buffer, walls) = (
        ctx.shader_system(),
        ctx.buffer_dynamic(),
        ctx.buffer_static(vec![].rect(30.0, [40.0, 40.0], [800.0 - 80.0, 600.0 - 80.0])),
    );

    let mut verts = vec![];
    'outer: loop {
        for res in engine.next().await.events {
            match res.event {
                shogo::Event::MouseClick(_mouse) => {
                    if res.element == button {
                        let _ = color_iter.next().unwrap_throw();
                    } else if res.element == shutdown_button {
                        break 'outer;
                    } else {
                        unreachable!();
                    }
                }
                shogo::Event::MouseMove(mouse) => {
                    if res.element == *canvas.as_ref() {
                        mouse_pos = convert_coord(res.element, mouse);
                    }
                }
            }
        }

        let radius = 30.0;
        let game_dim = [canvas.width() as f32, canvas.height() as f32];

        verts.clear();
        verts.line(radius, mouse_pos, [0.0, 0.0]);
        verts.line(radius, mouse_pos, game_dim);
        verts.line(radius, mouse_pos, [0.0, game_dim[1]]);
        verts.line(radius, mouse_pos, [game_dim[0], 0.0]);
        buffer.update(&verts);

        ctx.clear_color(0.13, 0.13, 0.13, 1.0);
        ctx.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

        draw_sys.draw_circles(
            &buffer,
            game_dim,
            color_iter.peek().unwrap_throw(),
            [0.0, 0.0],
            radius,
        );
        draw_sys.draw_squares(&walls, game_dim, &[1.0, 1.0, 1.0, 0.2], [0.0, 0.0], radius);
    }

    log!("all done!");
    */
}

fn convert_coord(canvas: web_sys::HtmlElement, e: web_sys::MouseEvent) -> [f32; 2] {
    let [x, y] = [e.client_x() as f32, e.client_y() as f32];
    let bb = canvas.get_bounding_client_rect();
    let tl = bb.x() as f32;
    let tr = bb.y() as f32;
    [x - tl, y - tr]
}
