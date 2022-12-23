//!
//! A simple webgl 2d drawing system that draws shapes using many small circles or squares.
//!
//! The data sent to the gpu is minimized by only sending the positions of the vertex.
//! The color can also be changed for all vertices in a buffer.
//!
//!
//!
use web_sys::WebGl2RenderingContext;
mod shader;

use shader::*;

pub use shader::Buffer;

const SQUARE_FRAG_SHADER_STR: &str = r#"#version 300 es
precision mediump float;
out vec4 out_color;
uniform vec4 bg;

void main() {
    //coord is between -0.5 and 0.5
    vec2 coord = gl_PointCoord - vec2(0.5,0.5);         
    out_color = bg;
}
"#;

const CIRCLE_FRAG_SHADER_STR: &str = r#"#version 300 es
precision mediump float;
out vec4 out_color;
uniform vec4 bg;

void main() {
    //coord is between -0.5 and 0.5
    vec2 coord = gl_PointCoord - vec2(0.5,0.5);
    float dissqr=dot(coord,coord);
    if(dissqr > 0.25){
        discard;
    }
    out_color = bg;    
}
"#;

const VERT_SHADER_STR: &str = r#"#version 300 es
in vec2 position;
uniform vec2 offset;
uniform mat3 mmatrix;
uniform float point_size;
void main() {
    gl_PointSize = point_size;
    vec3 pp=vec3(position.xy+offset,1.0);
    gl_Position = vec4((mmatrix*pp).xy,0.0, 1.0);
}
"#;

///
/// A buffer make with [`WebGl2RenderingContext::STATIC_DRAW`].
///
pub struct StaticBuffer<T>(Buffer<T>);

impl<T> std::ops::Deref for StaticBuffer<T> {
    type Target = Buffer<T>;
    fn deref(&self) -> &Buffer<T> {
        &self.0
    }
}

// impl StaticBuffer<[f32; 2]> {
//     fn new2<K>(
//         ctx: &WebGl2RenderingContext,
//         vec: &mut CpuBuffer<[f32; 2]>,
//         func: impl FnOnce(&mut ShapeBuilder)->K,
//     ) -> Result<(Self,K), String> {
//         vec.inner.clear();
//         let mut k = ShapeBuilder {
//             inner: &mut vec.inner,
//         };
//         let j=func(&mut k);
//         Self::new(ctx, &vec.inner).map(|a|(a,j))
//     }
// }

impl<T> StaticBuffer<T> {
    pub fn new(ctx: &WebGl2RenderingContext, verts: &[T]) -> Result<Self, String> {
        let mut buffer = StaticBuffer(Buffer::new(ctx)?);

        buffer.0.num_verts = verts.len();

        ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer.0.buffer));

        let n_bytes = verts.len() * std::mem::size_of::<T>();
        let points_buf: &[u8] =
            unsafe { std::slice::from_raw_parts(verts.as_ptr() as *const u8, n_bytes) };

        ctx.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            points_buf,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        Ok(buffer)
    }
}

///
/// A buffer make with [`WebGl2RenderingContext::DYNAMIC_DRAW`].
///
pub struct DynamicBuffer<T>(Buffer<T>);

impl<T> std::ops::Deref for DynamicBuffer<T> {
    type Target = Buffer<T>;
    fn deref(&self) -> &Buffer<T> {
        &self.0
    }
}

// impl DynamicBuffer<[f32; 2]> {
//     pub fn push_verts<K>(
//         &mut self,
//         vec: &mut CpuBuffer<[f32; 2]>,
//         func: impl FnOnce(&mut ShapeBuilder)->K,
//     ) ->K{
//         vec.inner.clear();
//         let mut k = ShapeBuilder {
//             inner: &mut vec.inner,
//         };
//         let j=func(&mut k);
//         self.update(&vec.inner);
//         j
//     }
// }
impl<T> DynamicBuffer<T> {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<Self, String> {
        Ok(DynamicBuffer(Buffer::new(ctx)?))
    }

    pub fn update_and_clear(&mut self,verts:&mut Vec<T>){
        self.update(verts);
        verts.clear();
    }
    pub fn update(&mut self, vertices: &[T]) {
        let ctx = &self.0.ctx;

        self.0.num_verts = vertices.len();

        ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.0.buffer));

        let n_bytes = vertices.len() * std::mem::size_of::<T>();
        let points_buf: &[u8] =
            unsafe { std::slice::from_raw_parts(vertices.as_ptr() as *const u8, n_bytes) };

        ctx.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            points_buf,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }
}

struct Args<'a> {
    pub verts: &'a Buffer<[f32; 2]>,
    pub primitive: u32,
    pub game_dim: [f32; 2],
    pub as_square: bool,
    pub color: &'a [f32; 4],
    pub offset: [f32; 2],
    pub point_size: f32,
}

// pub struct CpuBuffer<T> {
//     inner: Vec<T>,
// }
// impl<T> CpuBuffer<T> {
//     pub fn new() -> Self {
//         CpuBuffer { inner: vec![] }
//     }
// }

use wasm_bindgen::prelude::*;

///
/// Wrapper around a webgl2 context with convenience functions. Derefs to [`WebGl2RenderingContext`].
///
pub struct CtxWrap {
    pub ctx: WebGl2RenderingContext,
}

impl std::ops::Deref for CtxWrap {
    type Target = WebGl2RenderingContext;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl CtxWrap {
    pub fn new(a: &WebGl2RenderingContext) -> Self {
        CtxWrap { ctx: a.clone() }
    }

    // pub fn cpu_buffer<T>(&self) -> CpuBuffer<T> {
    //     CpuBuffer::new()
    // }
    ///
    /// Sets up alpha blending and disables depth testing.
    ///
    pub fn setup_alpha(&self) {
        self.disable(WebGl2RenderingContext::DEPTH_TEST);
        self.enable(WebGl2RenderingContext::BLEND);
        self.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
    }
    pub fn buffer_dynamic<T>(&self) -> DynamicBuffer<T> {
        DynamicBuffer::new(self).unwrap_throw()
    }

    // pub fn gpu_buffer_static<K>(
    //     &self,
    //     vec: &mut CpuBuffer<[f32; 2]>,
    //     func: impl FnOnce(&mut ShapeBuilder)->K,
    // ) -> Result<(StaticBuffer<[f32; 2]>,K), String> {
    //     StaticBuffer::new2(self, vec, func)
    // }


    pub fn buffer_static_and_clear(&self,a:&mut Vec<[f32;2]>)->StaticBuffer<[f32;2]>{
        let b=self.buffer_static(a);
        a.clear();
        b
    }

    pub fn buffer_static<T>(&self, a: &[T]) -> StaticBuffer<T> {
        StaticBuffer::new(self, a).unwrap_throw()
    }


    pub fn shader_system(&self) -> ShaderSystem {
        ShaderSystem::new(self).unwrap_throw()
    }
    pub fn draw_all(&self,color:[f32;4],func:impl FnOnce()){
        self.draw_clear(color);
        func();
        self.flush();
    }

    pub fn draw_clear(&self, color: [f32; 4]) {
        let [a, b, c, d] = color;
        self.ctx.clear_color(a, b, c, d);
        self.ctx
            .clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
    }
}

///
/// Primitive use to [`Shapes::rect`]
///
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl From<axgeom::Rect<f32>> for Rect {
    fn from(a: axgeom::Rect<f32>) -> Rect {
        Rect {
            x: a.x.start,
            y: a.y.start,
            w: a.x.end - a.x.start,
            h: a.y.end - a.y.start,
        }
    }
}

///
/// A simple shader program that allows the user to draw simple primitives.
///
pub struct ShaderSystem {
    circle_program: GlProgram,
    square_program: GlProgram,
    ctx: WebGl2RenderingContext,
}

impl Drop for ShaderSystem {
    fn drop(&mut self) {
        self.ctx.delete_program(Some(&self.circle_program.program));
        self.ctx.delete_program(Some(&self.square_program.program));
    }
}

impl ShaderSystem {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<ShaderSystem, String> {
        let circle_program = GlProgram::new(ctx, VERT_SHADER_STR, CIRCLE_FRAG_SHADER_STR)?;
        let square_program = GlProgram::new(ctx, VERT_SHADER_STR, SQUARE_FRAG_SHADER_STR)?;

        Ok(ShaderSystem {
            circle_program,
            square_program,
            ctx: ctx.clone(),
        })
    }

    fn draw(&mut self, args: Args) {
        let Args {
            verts,
            primitive,
            game_dim,
            as_square,
            color,
            offset,
            point_size,
        } = args;

        assert_eq!(verts.ctx, self.ctx);

        let scalex = 2.0 / game_dim[0];
        let scaley = 2.0 / game_dim[1];
        let tx = -1.0;
        let ty = 1.0;
        let matrix = [scalex, 0.0, 0.0, 0.0, -scaley, 0.0, tx, ty, 1.0];

        if as_square {
            self.square_program
                .draw(verts, primitive, offset, &matrix, point_size, color);
        } else {
            self.circle_program
                .draw(verts, primitive, offset, &matrix, point_size, color);
        };
    }

    ///
    /// when using [`View`],
    /// topleft corner maps to `[0,0]`
    /// borrom right maps to `dim`
    ///
    pub fn view(&mut self, game_dim: impl Into<[f32; 2]>, offset: impl Into<[f32; 2]>) -> View {
        View {
            sys: self,
            offset: offset.into(),
            dim: game_dim.into(),
        }
    }
}

///
/// A view to draw in. See [`ShaderSystem::view`]
///
pub struct View<'a> {
    sys: &'a mut ShaderSystem,
    offset: [f32; 2],
    dim: [f32; 2],
}
impl View<'_> {
    pub fn draw_squares(&mut self, verts: &Buffer<[f32; 2]>, point_size: f32, color: &[f32; 4]) {
        self.sys.draw(Args {
            verts,
            primitive: WebGl2RenderingContext::POINTS,
            game_dim: self.dim,
            as_square: true,
            color,
            offset: self.offset,
            point_size,
        })
    }
    pub fn draw_triangles(&mut self, verts: &Buffer<[f32; 2]>, color: &[f32; 4]) {
        self.sys.draw(Args {
            verts,
            primitive: WebGl2RenderingContext::TRIANGLES,
            game_dim: self.dim,
            as_square: true,
            color,
            offset: self.offset,
            point_size: 0.0,
        })
    }

    pub fn draw_circles(&mut self, verts: &Buffer<[f32; 2]>, point_size: f32, color: &[f32; 4]) {
        self.sys.draw(Args {
            verts,
            primitive: WebGl2RenderingContext::POINTS,
            game_dim: self.dim,
            as_square: false,
            color,
            offset: self.offset,
            point_size,
        })
    }
}

pub struct ShapeBuilder<'a> {
    inner:  &'a mut Vec<[f32; 2]>,
}

impl<'a> std::ops::Deref for ShapeBuilder<'a>{
    type Target=Vec<[f32;2]>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<'a> ShapeBuilder<'a> {
    pub fn clear(&mut self){
        self.inner.clear();
    }
    pub fn new(inner: &'a mut Vec<[f32;2]>)->Self{
        ShapeBuilder { inner }
    }
    pub fn points<I:IntoIterator<Item=[f32;2]>>(&mut self,it:I){
        self.inner.extend(it);
    }

    pub fn dot_line(&mut self, radius: f32, start: impl Into<[f32; 2]>, end: impl Into<[f32; 2]>) {
        let buffer = &mut self.inner;
        use axgeom::*;
        let start = Vec2::from(start.into());
        let end = Vec2::from(end.into());

        let offset = end - start;
        let dis_sqr = offset.magnitude2();
        let dis = dis_sqr.sqrt();

        let norm = offset / dis;

        let num = (dis / (radius)).floor() as usize;

        for i in 0..num {
            let pos = start + norm * (i as f32) * radius;
            buffer.push(pos.into());
        }
    }

    pub fn line(&mut self, radius: f32, start: impl Into<[f32; 2]>, end: impl Into<[f32; 2]>) {
        let buffer = &mut self.inner;
        use axgeom::*;
        let start = Vec2::from(start.into());
        let end = Vec2::from(end.into());

        let offset = end - start;
        let k = offset.rotate_90deg_right().normalize_to(1.0);
        let start1 = start + k * radius;
        let start2 = start - k * radius;

        let end1 = end + k * radius;
        let end2 = end - k * radius;

        let arr: [[f32; 2]; 6] = [
            start1.into(),
            start2.into(),
            end1.into(),
            start2.into(),
            end1.into(),
            end2.into(),
        ];

        buffer.extend(arr);
    }

    pub fn rect(&mut self, rect: impl Into<Rect>) {
        use axgeom::vec2;
        let rect: Rect = rect.into();

        let buffer = &mut self.inner;
        let start = vec2(rect.x, rect.y);
        let dim = vec2(rect.w, rect.h);

        let arr: [[f32; 2]; 6] = [
            start.into(),
            (start + vec2(dim.x, 0.0)).into(),
            (start + vec2(0.0, dim.y)).into(),
            (start + vec2(dim.x, 0.0)).into(),
            (start + dim).into(),
            (start + vec2(0.0, dim.y)).into(),
        ];

        buffer.extend(arr);
    }
}

///
/// Convert a mouse event to a coordinate for simple2d.
///
pub fn convert_coord(canvas: &web_sys::HtmlElement, e: &web_sys::MouseEvent) -> [f32; 2] {
    let rect = canvas.get_bounding_client_rect();

    let canvas_width: f64 = canvas
        .get_attribute("width")
        .unwrap_throw()
        .parse()
        .unwrap_throw();
    let canvas_height: f64 = canvas
        .get_attribute("height")
        .unwrap_throw()
        .parse()
        .unwrap_throw();

    let scalex = canvas_width / rect.width();
    let scaley = canvas_height / rect.height();

    let [x, y] = [e.client_x() as f64, e.client_y() as f64];

    let [x, y] = [(x - rect.left()) * scalex, (y - rect.top()) * scaley];
    [x as f32, y as f32]
}
