//!
//! A simple webgl drawing system that draws shapes using many small circles or squares.
//!

use web_sys::WebGl2RenderingContext;

use crate::circle_program::*;

pub use crate::circle_program::Buffer;

const SQUARE_FRAG_SHADER_STR: &str = r#"#version 300 es
precision mediump float;
out vec4 out_color;
uniform vec4 bg;


void main() {
    //coord is between -0.5 and 0.5
    vec2 coord = gl_PointCoord - vec2(0.5,0.5);
    /*
    float foo=coord.x*coord.x*coord.x*coord.x+coord.y*coord.y*coord.y*coord.y;
    if(foo > 0.25*0.25){                  //outside of circle radius?
        discard;
    } 
    */           
    out_color = bg;
}
"#;

const CIRCLE_FRAG_SHADER_STR: &str = r#"#version 300 es
precision mediump float;
out vec4 out_color;
uniform vec4 bg;
in float extra;

void main() {
    //coord is between -0.5 and 0.5
    vec2 coord = gl_PointCoord - vec2(0.5,0.5);
    float dissqr=dot(coord,coord);
    if(dissqr > 0.25){                  //outside of circle radius?
        discard;
    }

    /*
    vec2 rot=vec2(cos(extra),sin(extra));

    vec2 perpDir = vec2(rot.y, -rot.x);
    vec2 dirToPt1 = coord;
    float dist= abs(dot(normalize(perpDir), dirToPt1));
    
    
    if (dist<0.1 &&  dot(rot,coord)>0.0){
        out_color = vec4(0.0,0.0,0.0,1.0);
    }else{
        out_color = bg;
    }
    */
    out_color = bg;
    
    
}
"#;

const VERT_SHADER_STR: &str = r#"#version 300 es
in vec3 position;
out float extra;
uniform vec2 offset;
uniform mat3 mmatrix;
uniform float point_size;
void main() {
    gl_PointSize = point_size;
    vec3 pp=vec3(position.xy+offset,1.0);
    extra=position.z;
    gl_Position = vec4(mmatrix*pp, 1.0); //TODO optimize
}
"#;

pub struct StaticBuffer(Buffer);

impl std::ops::Deref for StaticBuffer {
    type Target = Buffer;
    fn deref(&self) -> &Buffer {
        &self.0
    }
}

impl StaticBuffer {
    pub fn new(ctx: &WebGl2RenderingContext, verts: &[[f32; 2]]) -> Result<Self, String> {
        let mut buffer = StaticBuffer(Buffer::new(ctx)?);

        buffer.0.num_verts = verts.len();

        ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer.0.buffer));

        let n_bytes = verts.len() * std::mem::size_of::<[f32; 2]>();
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

pub struct DynamicBuffer(Buffer);

impl std::ops::Deref for DynamicBuffer {
    type Target = Buffer;
    fn deref(&self) -> &Buffer {
        &self.0
    }
}

impl DynamicBuffer {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<Self, String> {
        Ok(DynamicBuffer(Buffer::new(ctx)?))
    }
    pub fn update(&mut self, vertices: &[[f32; 2]]) {
        let ctx = &self.0.ctx;

        self.0.num_verts = vertices.len();

        ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.0.buffer));

        let n_bytes = vertices.len() * std::mem::size_of::<[f32; 2]>();
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
    pub primitive: u32,
    pub game_dim: [f32; 2],
    pub as_square: bool,
    pub color: &'a [f32; 4],
    pub offset: [f32; 2],
    pub point_size: f32,
}

use wasm_bindgen::prelude::*;

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
    pub fn setup_alpha(&self) {
        self.disable(WebGl2RenderingContext::DEPTH_TEST);
        self.enable(WebGl2RenderingContext::BLEND);
        self.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
    }
    pub fn buffer_dynamic(&self) -> DynamicBuffer {
        DynamicBuffer::new(self).unwrap_throw()
    }
    pub fn buffer_static(&self, a: &[[f32; 2]]) -> StaticBuffer {
        StaticBuffer::new(self, a).unwrap_throw()
    }
    pub fn shader_system(&self) -> ShaderSystem {
        ShaderSystem::new(self).unwrap_throw()
    }

    pub fn draw_clear(&self, color: [f32; 4]) {
        let [a, b, c, d] = color;
        self.ctx.clear_color(a, b, c, d);
        self.ctx
            .clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
    }
}

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

    pub fn camera(&mut self, game_dim: impl Into<[f32; 2]>, offset: impl Into<[f32; 2]>) -> Camera {
        Camera {
            sys: self,
            offset: offset.into(),
            dim: game_dim.into(),
        }
    }
}

pub struct Camera<'a> {
    sys: &'a mut ShaderSystem,
    offset: [f32; 2],
    dim: [f32; 2],
}
impl Camera<'_> {
    pub fn draw_squares(&mut self, verts: &Buffer, point_size: f32, color: &[f32; 4]) {
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
    pub fn draw_triangles(&mut self, verts: &Buffer, color: &[f32; 4]) {
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

    pub fn draw_circles(&mut self, verts: &Buffer, point_size: f32, color: &[f32; 4]) {
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

pub trait Shapes {
    fn line(&mut self, width: f32, start: impl Into<[f32; 2]>, end: impl Into<[f32; 2]>);
    fn dot_line(&mut self, radius: f32, start: impl Into<[f32; 2]>, end: impl Into<[f32; 2]>);

    fn rect(&mut self, rect: impl Into<Rect>);
}
impl Shapes for Vec<[f32; 2]> {
    fn dot_line(&mut self, radius: f32, start: impl Into<[f32; 2]>, end: impl Into<[f32; 2]>) {
        let buffer = self;
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

    fn line(&mut self, radius: f32, start: impl Into<[f32; 2]>, end: impl Into<[f32; 2]>) {
        let buffer = self;
        use axgeom::*;
        let start = Vec2::from(start.into());
        let end = Vec2::from(end.into());

        /*
        let offset = end - start;
        let dis_sqr = offset.magnitude2();
        let dis = dis_sqr.sqrt();

        let norm = offset / dis;



        let num = (dis / (radius)).floor() as usize;

        for i in 0..num {
            let pos = start + norm * (i as f32) * radius;
            buffer.push(pos.into());
        }
        */

        let offset = end - start;
        let k = offset.rotate_90deg_right().normalize_to(1.0);
        let start1 = start + k * radius;
        let start2 = start - k * radius;

        let end1 = end + k * radius;
        let end2 = end - k * radius;

        //let arr = [start1, start2, end1, start2, end1, end2];

        buffer.push(start1.into());
        buffer.push(start2.into());
        buffer.push(end1.into());
        buffer.push(start2.into());
        buffer.push(end1.into());
        buffer.push(end2.into());
    }

    fn rect(&mut self, rect: impl Into<Rect>) {
        use axgeom::vec2;
        let rect: Rect = rect.into();

        let buffer = self;
        let start = vec2(rect.x, rect.y);
        let dim = vec2(rect.w, rect.h);

        buffer.push(start.into());
        buffer.push((start + vec2(dim.x, 0.0)).into());
        buffer.push((start + vec2(0.0, dim.y)).into());

        buffer.push((start + vec2(dim.x, 0.0)).into());
        buffer.push((start + dim).into());
        buffer.push((start + vec2(0.0, dim.y)).into());
    }
}
