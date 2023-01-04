//!
//! A simple webgl 2d drawing system that draws shapes using many small circles or squares.
//!
//! The data sent to the gpu is minimized by only sending the positions of the vertex.
//! The color can also be changed for all vertices in a buffer.
//!
//!
//!
use gloo::console::log;
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
in vec3 position;
uniform mat4 mmatrix;
uniform float point_size;
void main() {
    gl_PointSize = point_size;
    vec4 pp=vec4(position,1.0);
    vec4 j = mmatrix*pp;
    gl_Position = j;
}
"#;




///
/// A buffer make with [`WebGl2RenderingContext::STATIC_DRAW`].
///
pub struct StaticBuffer(Buffer);


//TODO why needed?
impl std::ops::Deref for StaticBuffer {
    type Target = Buffer;
    fn deref(&self) -> &Buffer {
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

pub type Vertex = [f32; 3];

impl StaticBuffer {
    pub fn new(ctx: &WebGl2RenderingContext, verts: &[Vertex]) -> Result<Self, String> {
        let mut buffer = StaticBuffer(Buffer::new(ctx)?);

        buffer.0.num_verts = verts.len();

        ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer.0.buffer));

        let n_bytes = verts.len() * std::mem::size_of::<Vertex>();
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


pub struct IndexBuffer(Buffer);
impl IndexBuffer{
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<Self, String> {
        Ok(IndexBuffer(Buffer::new(ctx)?))
    }
    pub fn update(&mut self, vertices: &[u16]) {
        let ctx = &self.0.ctx;

        self.0.num_verts = vertices.len();

        ctx.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.0.buffer));

        
        let n_bytes = vertices.len() * 2;
        let points_buf: &[u8] =
            unsafe { std::slice::from_raw_parts(vertices.as_ptr() as *const u8, n_bytes) };


        ctx.buffer_data_with_u8_array(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            points_buf,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }
}


///
/// A buffer make with [`WebGl2RenderingContext::DYNAMIC_DRAW`].
///
pub struct DynamicBuffer(Buffer);

impl std::ops::Deref for DynamicBuffer {
    type Target = Buffer;
    fn deref(&self) -> &Buffer {
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

impl DynamicBuffer {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<Self, String> {
        Ok(DynamicBuffer(Buffer::new(ctx)?))
    }

    pub fn update_clear(&mut self, verts: &mut Vec<Vertex>) {
        self.update_no_clear(verts);
        verts.clear();
    }
    // pub fn update_from_bytes(&mut self, vertices: &[u8]) {
    //     assert_eq!(vertices.len() % (4*3),0);

    //     let ctx = &self.0.ctx;

    //     self.0.num_verts = vertices.len()/(4*3);

    //     ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.0.buffer));

    //     ctx.buffer_data_with_u8_array(
    //         WebGl2RenderingContext::ARRAY_BUFFER,
    //         vertices,
    //         WebGl2RenderingContext::DYNAMIC_DRAW,
    //     );

    // }
    pub fn update_no_clear(&mut self, vertices: &[Vertex]) {
        let ctx = &self.0.ctx;

        self.0.num_verts = vertices.len();

        ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.0.buffer));

        let n_bytes = vertices.len() * std::mem::size_of::<Vertex>();
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
    pub verts: &'a Buffer,
    pub indexes:Option<&'a IndexBuffer>,
    pub primitive: u32,
    //pub game_dim: [f32; 2],
    //pub as_square: bool,
    pub color: &'a [f32; 4],
    //pub offset: [f32; 2],
    pub matrix:&'a [f32;16],
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

pub fn ctx_wrap(a: &WebGl2RenderingContext) -> CtxWrap {
    CtxWrap::new(a)
}
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
        self.enable(WebGl2RenderingContext::DEPTH_TEST);
        self.enable(WebGl2RenderingContext::BLEND);
        self.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
    }
    pub fn buffer_dynamic(&self) -> DynamicBuffer {
        DynamicBuffer::new(self).unwrap_throw()
    }

    pub fn buffer_static_clear(&self, a: &mut Vec<Vertex>) -> StaticBuffer {
        let b = self.buffer_static_no_clear(a);
        a.clear();
        b
    }

    pub fn buffer_static_no_clear(&self, a: &[Vertex]) -> StaticBuffer {
        StaticBuffer::new(self, a).unwrap_throw()
    }

    pub fn shader_system(&self) -> ShaderSystem {

        ShaderSystem::new(self).map_err(|e|{
            log!(format!("{:?}",e));
            e
        }).unwrap_throw()
    }
    // pub fn draw_all(&self, color: [f32; 4], func: impl FnOnce()) {
    //     self.draw_clear(color);
    //     func();
    //     self.flush();
    // }

    pub fn draw_clear(&self, color: [f32; 4]) {
        let [a, b, c, d] = color;
        self.ctx.clear_color(a, b, c, d);
        self.ctx
            .clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT | web_sys::WebGl2RenderingContext::DEPTH_BUFFER_BIT);
    }
}

///
/// Primitive use to [`ShapeBuilder::rect`]
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
    //circle_program: GlProgram,
    square_program: GlProgram,
    ctx: WebGl2RenderingContext
}

impl Drop for ShaderSystem {
    fn drop(&mut self) {
        //self.ctx.delete_program(Some(&self.circle_program.program));
        self.ctx.delete_program(Some(&self.square_program.program));
    }
}

impl ShaderSystem {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<ShaderSystem, String> {
        //let circle_program = GlProgram::new(ctx, VERT_SHADER_STR, CIRCLE_FRAG_SHADER_STR)?;
        let square_program = GlProgram::new(ctx, VERT_SHADER_STR, SQUARE_FRAG_SHADER_STR)?;

        Ok(ShaderSystem {
            //circle_program,
            square_program,
            ctx: ctx.clone()
        })
    }

    fn draw(&mut self, args: Args) {
        let Args {
            verts,
            primitive,
            matrix,
            indexes,
            color,
            point_size,
        } = args;

        assert_eq!(verts.ctx, self.ctx);


        //if as_square {
            self.square_program
                .draw(indexes,verts, primitive, &matrix, point_size, color);
        // } else {
        //     self.circle_program
        //         .draw(verts, primitive, &matrix, point_size, color);
        // };
    }

    ///
    /// when using [`View`],
    /// topleft corner maps to `[0,0]`
    /// borrom right maps to `dim`
    ///
    pub fn view<'a>(&'a mut self, matrix:&'a [f32;16]) -> View<'a> {
        View {
            sys: self,
            matrix
        }
    }
}

///
/// A view to draw in. See [`ShaderSystem::view`]
///
pub struct View<'a> {
    sys: &'a mut ShaderSystem,
    matrix:&'a [f32;16],
    // offset: [f32; 2],
    // dim: [f32; 2],
}
impl View<'_> {
    // pub fn draw_squares(&mut self, verts: &Buffer, point_size: f32, color: &[f32; 4]) {
    //     self.sys.draw(Args {
    //         verts,
    //         primitive: WebGl2RenderingContext::POINTS,
    //         matrix:self.matrix,
    //         color,
    //         point_size,
    //     })
    // }
    pub fn draw_triangles(&mut self, verts: &Buffer,indexes:Option<&IndexBuffer>, color: &[f32; 4]) {
        self.sys.draw(Args {
            verts,
            primitive: WebGl2RenderingContext::TRIANGLES,
            matrix: self.matrix,
            indexes,
            color,
            point_size: 0.0,
        })
    }

    // pub fn draw_circles(&mut self, verts: &Buffer, point_size: f32, color: &[f32; 4]) {
    //     self.sys.draw(Args {
    //         verts,
    //         primitive: WebGl2RenderingContext::POINTS,
    //         matrix:self.matrix,
    //         as_square: false,
    //         color,
    //         point_size,
    //     })
    // }
}

pub fn shapes(a: &mut Vec<Vertex>) -> ShapeBuilder {
    ShapeBuilder::new(a)
}
pub struct ShapeBuilder<'a> {
    inner: &'a mut Vec<Vertex>,
}

impl<'a> std::ops::Deref for ShapeBuilder<'a> {
    type Target = Vec<Vertex>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<'a> ShapeBuilder<'a> {
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    pub fn new(inner: &'a mut Vec<Vertex>) -> Self {
        ShapeBuilder { inner }
    }

    pub fn dot_line(
        &mut self,
        radius: f32,
        start: impl Into<[f32; 2]>,
        end: impl Into<[f32; 2]>,
    ) -> &mut Self {
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
            buffer.push([pos.x,pos.y,0.0]);
        }
        self
    }

    pub fn line(
        &mut self,
        radius: f32,
        start: impl Into<[f32; 2]>,
        end: impl Into<[f32; 2]>,
    ) -> &mut Self {
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

        let arr: [Vertex; 6] = [
            [start1.x,start1.y,0.0],
            [start2.x,start2.y,0.0],
            [end1.x,end1.y,0.0],
            [start2.x,start2.y,0.0],
            [end1.x,end1.y,0.0],
            [end2.x,end2.y,0.0],
        ];

        buffer.extend(arr);
        self
    }

    pub fn rect(&mut self, rect: impl Into<Rect>,depth:f32) -> &mut Self {
        use axgeom::vec2;
        let rect: Rect = rect.into();

        let buffer = &mut self.inner;
        let start = vec2(rect.x, rect.y);
        let dim = vec2(rect.w, rect.h);

        let arr: [Vertex; 6] = [
            to_vertex(start,depth),
            to_vertex(start + vec2(dim.x, 0.0),depth),
            to_vertex(start + vec2(0.0, dim.y),depth),
            to_vertex(start + vec2(dim.x, 0.0),depth),
            to_vertex(start + dim,depth),
            to_vertex(start + vec2(0.0, dim.y),depth),
        ];

        buffer.extend(arr);
        self
    }
}
fn to_vertex(a:axgeom::Vec2<f32>,depth:f32)->Vertex{
    [a.x,a.y,depth]
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
