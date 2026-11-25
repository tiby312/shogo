use futures::{select, FutureExt};
use glam::DVec2;
use gloo::console::log;
use serde::{Deserialize, Serialize};
use shogo::utils;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlImageElement, MouseEvent};

///Common data sent from the main thread to the worker.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MEvent {
    CanvasMouseMove { x: f64, y: f64 },
    ButtonClick,
    ShutdownClick,
}
use std::f64::consts::PI;

#[wasm_bindgen]
pub async fn main_entry() {
    use futures::StreamExt;

    log!("demo start");

    let (canvas, button, shutdown_button) = (
        utils::get_by_id_canvas("mycanvas"),
        utils::get_by_id_elem("mybutton"),
        utils::get_by_id_elem("shutdownbutton"),
    );

    let ctx = canvas.get_context("2d").unwrap_throw().unwrap_throw();
    let ctx = ctx
        .dyn_ref::<web_sys::CanvasRenderingContext2d>()
        .unwrap_throw();

    let (tx, mut rx) = futures::channel::mpsc::unbounded();

    let _e = [
        shogo::reg(&tx, &canvas, "mousemove", move |e| {
            let e: &MouseEvent = e.dyn_ref().unwrap_throw();
            let x = e.x() as f64;
            let y = e.y() as f64;
            Some(MEvent::CanvasMouseMove { x, y })
        }),
        shogo::reg(&tx, &button, "click", move |_| Some(MEvent::ButtonClick)),
        shogo::reg(&tx, &shutdown_button, "click", move |_| {
            Some(MEvent::ShutdownClick)
        }),
    ];

    let mut ship_pos = DVec2::new(500.0, 500.0);
    let mut ship_vel = DVec2::new(0.0, 0.0);
    let _rotation = 0.0;
    let mut mouse_pos = DVec2::new(0.0, 0.0);
    let _last_mouse_pos = DVec2::new(0.0, 0.0);
    let mut ship_rot = 0.0f64;

    let mut going_forwards = true;

    // 3. Instantiate a new Image element
    let image = HtmlImageElement::new().unwrap_throw();
    image.set_src("background.png");

    let mut rr = shogo::RequestAnimationFrameMan::new();
    rr.request_animation_frame();

    loop {
        let dt = loop {
            select! {
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
        let offset = mouse_pos - middle;

        let inner_ring = Ring { radius: 40.0 };

        let outer_ring = Ring { radius: 100.0 };

        let shooting_ring = Ring { radius: 200.0 };

        if inner_ring.is_outside(offset.length()) {
            let target_angle = offset.y.atan2(offset.x);

            // 3. Find the shortest angular distance between current and target angle
            // This prevents the ship from spinning the long way around
            let mut angle_diff1 = target_angle - ship_rot;

            // Normalize the angle difference to the range [-PI, PI]
            angle_diff1 = (angle_diff1 + PI).rem_euclid(2.0 * PI) - PI;

            let mut angle_diff2 = target_angle - ship_rot + PI;

            // Normalize the angle difference to the range [-PI, PI]
            angle_diff2 = (angle_diff2 + PI).rem_euclid(2.0 * PI) - PI;

            // 4. Calculate maximum rotation allowed this frame
            let max_rotation = 0.002 * dt;

            let rotation_step = if angle_diff1.abs() <= angle_diff2.abs() {
                log!("f");
                going_forwards = true;
                angle_diff1
            } else {
                log!("b");
                going_forwards = false;
                angle_diff2
            }
            .clamp(-max_rotation, max_rotation);

            // 6. Apply the rotation
            ship_rot += rotation_step;
        }

        if going_forwards {
            if outer_ring.is_outside(offset.length()) {
                let k = DVec2::new(ship_rot.cos(), ship_rot.sin());
                ship_vel += k * 0.0005 * dt;
            }
        } else {
            if outer_ring.is_outside(offset.length()) {
                let k = DVec2::new(ship_rot.cos(), ship_rot.sin());
                ship_vel -= k * 0.0005 * dt;
            }
        }

        ship_pos += ship_vel * dt;

        ctx.save();
        // Center the camera on the spaceship
        ctx.translate(
            canvas.width() as f64 / 2. - ship_pos.x,
            canvas.height() as f64 / 2. - ship_pos.y,
        )
        .unwrap_throw();

        ctx.draw_image_with_html_image_element_and_dw_and_dh(&image, 0.0, 0.0, 2000.0, 2000.0)
            .unwrap_throw();

        ctx.set_fill_style_str("white");
        draw_triangle(&ctx, ship_pos, ship_rot);

        ctx.restore();

        let circle = |x, y, radius| {
            ctx.begin_path();
            ctx.arc(x, y, radius, 0.0, std::f64::consts::PI * 2.0)
                .unwrap_throw();
            ctx.set_line_width(1.0);
            ctx.set_stroke_style_str("white");
            ctx.stroke();
        };

        circle(middle.x, middle.y, inner_ring.radius);
        circle(middle.x, middle.y, outer_ring.radius);
        circle(middle.x, middle.y, shooting_ring.radius);

        rr.request_animation_frame();
    }
}

fn draw_triangle(ctx: &web_sys::CanvasRenderingContext2d, center: DVec2, rotation: f64) {
    ctx.save();
    ctx.translate(center.x, center.y).unwrap_throw();
    ctx.rotate(rotation).unwrap_throw();
    ctx.begin_path();
    ctx.move_to(-10.0, -25.0);
    ctx.line_to(25.0, 0.0);
    ctx.line_to(-10.0, 25.0);
    ctx.close_path();
    ctx.fill();
    ctx.restore();
}

struct Ring {
    radius: f64,
}

impl Ring {
    fn is_outside(&self, length: f64) -> bool {
        length * length >= self.radius * self.radius
    }
}
