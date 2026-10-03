use std::{any, borrow::Cow, cell::RefCell, rc::Rc};

use futures::{FutureExt, SinkExt, channel::mpsc::{UnboundedReceiver, UnboundedSender}, select};
use glam::{DVec2, Vec2};
use gloo::{console::log, net::http::Request};
use serde::{Deserialize, Serialize};
use shogo::utils;
use wasm_bindgen::prelude::*;
use web_sys::{Event, EventTarget, HtmlImageElement, MouseEvent};
use gloo::render::{request_animation_frame, AnimationFrame};

const COLORS: &[[f32; 4]] = &[
    [1.0, 0.0, 0.0, 0.5],
    [0.0, 1.0, 0.0, 0.5],
    [0.0, 0.0, 1.0, 0.5],
];

///Common data sent from the main thread to the worker.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MEvent {
    CanvasMouseMove { x: f64, y: f64 },
    ButtonClick,
    ShutdownClick,
}
use std::f64::consts::PI;
            

#[test]
fn test(){
    let angle_diff=180.0;
    let g=(angle_diff + PI).rem_euclid(2.0 * PI) - PI;
    dbg!(g);
    panic!();
}

#[wasm_bindgen]
pub async fn main_entry() {
    use futures::StreamExt;

    log!("demo start");

    let (canvas, button, shutdown_button) = (
        utils::get_by_id_canvas("mycanvas"),
        utils::get_by_id_elem("mybutton"),
        utils::get_by_id_elem("shutdownbutton"),
    );

    let ctx=canvas.get_context("2d").unwrap_throw().unwrap_throw();
    let ctx=ctx.dyn_ref::<web_sys::CanvasRenderingContext2d>().unwrap_throw();

    

    
    let (tx,mut rx)=futures::channel::mpsc::unbounded();
 

    let _e=[
        shogo::reg(&tx, &canvas, "mousemove", move | e| {
            let e:&MouseEvent = e.dyn_ref().unwrap_throw();
            let x=e.x() as f64;
            let y = e.y() as f64;
            Some(MEvent::CanvasMouseMove { x, y })
        }),
        shogo::reg(&tx, &button, "click", move | _| {
            Some(MEvent::ButtonClick)
        }),
        shogo::reg(&tx, &shutdown_button, "click", move | _| {
            Some(MEvent::ShutdownClick)
        })
    ];
    



    let mut ship_pos=DVec2::new(500.0, 500.0);
    let mut ship_vel=DVec2::new(0.0, 0.0);
    let mut rotation=0.0;
    let mut mouse_pos=DVec2::new(0.0, 0.0);
    let mut last_mouse_pos=DVec2::new(0.0, 0.0);
    let mut ship_rot=0.0f64;

    let mut going_forwards=true;
 
    // 3. Instantiate a new Image element
    let image = HtmlImageElement::new().unwrap_throw();
    image.set_src("background.png");

    let mut rr=shogo::RequestAnimationFrameMan::new();
    rr.request_animation_frame();
  

    loop{
        let dt=loop{
            select!{
                timestamp = rr.next().fuse() => {
                    //log!("received frame delta: {}", timestamp.delta());
                    break timestamp.delta();
                }
                e = rx.next() => {
                    //log!("received event: {}", format!("{:?}", e));
                    match e.unwrap_throw()
                    {
                        MEvent::CanvasMouseMove { x, y } => {
                            mouse_pos=DVec2::new(x , y );
                        },
                        MEvent::ButtonClick => {
                            // Handle button click event if needed
                        },
                        MEvent::ShutdownClick => {
                            // Handle shutdown click event if needed
                        }
                    }
                }
            }
        };
        

        let middle = DVec2::new(canvas.width() as f64 / 2.0, canvas.height() as f64 / 2.0);
        let offset = mouse_pos-middle;
        let last_offset=last_mouse_pos-middle;

        
        let forward_fan=InfiniteFan2D{
            origin: middle,
            direction: DVec2::from_angle(ship_rot),
            angle_radians: 2.0, // Example angle, adjust as needed
        };

        let backward_fan=InfiniteFan2D{
            origin: middle,
            direction: DVec2::from_angle(ship_rot+std::f64::consts::PI),
            angle_radians: 2.0, // Example angle, adjust as needed
        };


        

        let inner_ring=Ring{
            radius:40.0
        };

        let outer_ring=Ring { radius: 100.0 };


        if inner_ring.is_outside(offset.length()) && inner_ring.is_inside(last_offset.length())
        {
            log!("inside ring");
            if forward_fan.contains_point(mouse_pos){
                //we entered the inner ring from the forward fan
                going_forwards=true;
                log!("switching to forward");
            }else if backward_fan.contains_point(mouse_pos){
                //we entered the inner ring from the backward fan
                going_forwards=false;
                log!("switching to backward");
            }
        }


        if going_forwards{
            if forward_fan.contains_point(mouse_pos) && outer_ring.is_outside(offset.length()){
                let k=DVec2::new(ship_rot.cos(), ship_rot.sin());
                ship_vel+=k*0.0005*dt;
                
            }
        }else{
            if backward_fan.contains_point(mouse_pos) && outer_ring.is_outside(offset.length()){
                let k=DVec2::new(ship_rot.cos(), ship_rot.sin());
                ship_vel-=k*0.0005*dt;
            }
        }

        ship_pos += ship_vel * dt;
        
        


        if inner_ring.is_outside(offset.length())
        {

            let ee=if going_forwards{
                0.0
            }else{
                std::f64::consts::PI
            };

            let target_angle = offset.y.atan2(offset.x);

            // 3. Find the shortest angular distance between current and target angle
            // This prevents the ship from spinning the long way around
            let mut angle_diff = target_angle - (ship_rot+ee);
            
            use std::f64::consts::PI;
            // Normalize the angle difference to the range [-PI, PI]
            angle_diff = (angle_diff + PI).rem_euclid(2.0 * PI) - PI;

            // 4. Calculate maximum rotation allowed this frame
            let max_rotation = 0.002 * dt;

            // 5. Clamp the rotation step so it doesn't overshoot
            let rotation_step = angle_diff.clamp(-max_rotation, max_rotation);

            // 6. Apply the rotation
            ship_rot += rotation_step;


        }
        


       
        
        ctx.save();
        // Center the camera on the spaceship
        ctx.translate(canvas.width() as f64/ 2. - ship_pos.x, canvas.height() as f64 / 2. - ship_pos.y).unwrap_throw();
  
        ctx.draw_image_with_html_image_element_and_dw_and_dh(&image, 0.0, 0.0,2000.0, 2000.0).unwrap_throw();
        
        

        ctx.set_fill_style_str("white");
        draw_triangle(&ctx,ship_pos,ship_rot);






        ctx.restore();

        
        let line=|x1, y1, x2, y2| {
            ctx.begin_path();
            ctx.move_to(x1, y1);
            ctx.line_to(x2, y2);
            ctx.stroke();
        };

        let rot=DVec2::from_angle(ship_rot+1.0);
        line(middle.x, middle.y, middle.x+rot.x*500.0, middle.y+rot.y*500.0);

        let rot=DVec2::from_angle(ship_rot-1.0);
        line(middle.x, middle.y, middle.x+rot.x*500.0, middle.y+rot.y*500.0);


        let rot=DVec2::from_angle(ship_rot+1.0+PI);
        line(middle.x, middle.y, middle.x+rot.x*500.0, middle.y+rot.y*500.0);

        let rot=DVec2::from_angle(ship_rot-1.0+PI);
        line(middle.x, middle.y, middle.x+rot.x*500.0, middle.y+rot.y*500.0);


        let circle=|x,y,radius|{
            ctx.begin_path();
            ctx.arc(x, y, radius, 0.0, std::f64::consts::PI * 2.0).unwrap_throw();
            ctx.set_line_width(1.0);
            ctx.set_stroke_style_str("white");
            ctx.stroke();
        };

        circle(middle.x, middle.y, inner_ring.radius);
        circle(middle.x, middle.y, outer_ring.radius);
        
        
        rr.request_animation_frame();

        last_mouse_pos = mouse_pos;


    }
}

fn draw_triangle(ctx: &web_sys::CanvasRenderingContext2d, center: DVec2, rotation: f64) {
    ctx.save();
    ctx.translate(center.x , center.y).unwrap_throw();
    ctx.rotate(rotation).unwrap_throw();
    ctx.begin_path();
    ctx.move_to(-10.0, -25.0);
    ctx.line_to(25.0, 0.0);
    ctx.line_to(-10.0, 25.0);
    ctx.close_path();
    ctx.fill();
    ctx.restore();
}

pub struct InfiniteFan2D {
    pub origin: DVec2,
    /// Must be a normalized vector pointing in the center direction of the fan
    pub direction: DVec2,
    /// The total field of view angle of the fan in radians
    pub angle_radians: f64,
}

impl InfiniteFan2D {
    pub fn contains_point(&self, point: DVec2) -> bool {
        // 1. Get vector from origin to target point
        let to_point = point - self.origin;

        // Handle the edge case where the point is exactly on the origin
        if to_point == DVec2::ZERO {
            return true; 
        }

        // 2. Normalize the target vector
        let to_point_dir = to_point.normalize();

        // 3. Calculate dot product (cosine of the angle between them)
        let dot = self.direction.dot(to_point_dir);

        // 4. Compare against the cosine of half the fan angle
        let half_angle_cos = (self.angle_radians * 0.5).cos();

        // If the dot product is higher, the angle is smaller, meaning it's inside
        dot >= half_angle_cos
    }
}


        struct Ring{
            radius:f64
        }

        impl Ring{
            fn is_inside(&self, length:f64) -> bool {
                length * length < self.radius * self.radius
            }
            fn is_outside(&self, length: f64) -> bool {
                length * length >= self.radius * self.radius
            }
        }
        


// pub fn point_in_triangle(p: DVec2, a: DVec2, b: DVec2, c: DVec2) -> bool {
//     let v0 = c - a;
//     let v1 = b - a;
//     let v2 = p - a;

//     let dot00 = v0.dot(v0);
//     let dot01 = v0.dot(v1);
//     let dot02 = v0.dot(v2);
//     let dot11 = v1.dot(v1);
//     let dot12 = v1.dot(v2);

//     // Compute barycentric coordinates
//     let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
//     let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
//     let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

//     // Check if point is in triangle
//     (u >= 0.0) && (v >= 0.0) && (u + v <= 1.0)
// }

// #[wasm_bindgen]
// pub async fn worker_entry() {
//     use shogo::simple2d;

//     let (mut w, ss) = shogo::EngineWorker::new().await;
//     let mut frame_timer = shogo::FrameTimer::new(30, ss);

//     let canvas = w.canvas();
//     let ctx = simple2d::ctx_wrap(&utils::get_context_webgl2_offscreen(&canvas));

//     //TODO put this in the library
//     ctx.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

//     let mut draw_sys = ctx.shader_system();
//     let mut buffer = ctx.buffer_dynamic();
//     let cache = &mut vec![];
//     simple2d::shapes(cache).rect(simple2d::Rect {
//         x: 40.0,
//         y: 40.0,
//         w: 800.0 - 80.0,
//         h: 600.0 - 80.0,
//     });
//     let walls = ctx.buffer_static_clear(cache);

//     ctx.setup_alpha();

//     // setup game data
//     let mut mouse_pos = [0.0f32; 2];
//     let mut color_iter = COLORS.iter().cycle().peekable();
//     let radius = 4.0;
//     let game_dim = [canvas.width() as f32, canvas.height() as f32];

//     'outer: loop {
//         for e in frame_timer.next().await {
//             match e {
//                 MEvent::CanvasMouseMove { x, y } => mouse_pos = [*x, *y],
//                 MEvent::ButtonClick => {
//                     let _ = color_iter.next();
//                 }
//                 MEvent::ShutdownClick => break 'outer,
//             }
//         }

//         simple2d::shapes(cache)
//             .line(radius, mouse_pos, [0.0, 0.0])
//             .line(radius, mouse_pos, game_dim)
//             .line(radius, mouse_pos, [0.0, game_dim[1]])
//             .line(radius, mouse_pos, [game_dim[0], 0.0]);

//         buffer.update_clear(cache);

//         ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

//         let matrix = projection(game_dim, [0.0, 0.0]);

//         let mut v = draw_sys.view(&matrix);
//         v.draw_triangles(&walls, &[1.0, 1.0, 1.0, 0.2]);
//         v.draw_triangles(&buffer, color_iter.peek().unwrap_throw());

//         ctx.flush();
//     }

//     w.post_message(());

//     log!("worker thread closing");
// }

// fn convert_coord(canvas: &web_sys::HtmlElement, event: &web_sys::Event) -> [f32; 2] {
//     use wasm_bindgen::JsCast;
//     shogo::simple2d::convert_coord(canvas, event.dyn_ref().unwrap_throw())
// }

// fn projection(dim: [f32; 2], offset: [f32; 2]) -> [f32; 16] {
//     let scale = |scalex, scaley, scalez| {
//         [
//             scalex, 0., 0., 0., 0., scaley, 0., 0., 0., 0., scalez, 0., 0., 0., 0., 1.0,
//         ]
//     };

//     let translation = |tx, ty, tz| {
//         [
//             1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., tx, ty, tz, 1.,
//         ]
//     };

//     let x_rotation = |angle_rad: f32| {
//         let c = angle_rad.cos();
//         let s = angle_rad.sin();

//         [1., 0., 0., 0., 0., c, s, 0., 0., -s, c, 0., 0., 0., 0., 1.]
//     };

//     let y_rotation = |angle_rad: f32| {
//         let c = angle_rad.cos();
//         let s = angle_rad.sin();

//         [c, 0., -s, 0., 0., 1., 0., 0., s, 0., c, 0., 0., 0., 0., 1.]
//     };

//     let z_rotation = |angle_rad: f32| {
//         let c = angle_rad.cos();
//         let s = angle_rad.sin();

//         [c, s, 0., 0., -s, c, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.]
//     };

//     use webgl_matrix::prelude::*;

//     let mut id = Mat4::identity();

//     let az = &translation(-dim[0] / 2. + offset[0], -dim[1] / 2. + offset[1], 0.0);
//     let t = &translation(-1.0, 1.0, 0.0);
//     let a1 = &scale(2.0, -2.0, 0.0);
//     let a2 = &scale(1.0 / dim[0], 1.0 / dim[1], 0.0);
//     id.mul(az).mul(a1).mul(a2).mul(&t);
//     id
// }
