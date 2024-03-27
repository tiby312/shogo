//!
//! A simple webgl 2d drawing system that draws shapes using many small circles or squares.
//!
//! The data sent to the gpu is minimized by only sending the positions of the vertex.
//! The color can also be changed for all vertices in a buffer.
//!
use gloo::console::log;
use web_sys::WebGl2RenderingContext;
mod shader;

use shader::*;

pub use shader::Buffer;



const SQUARE_FRAG_SHADER_STR: &str = r#"#version 300 es
precision mediump float;
out vec4 out_color;
//uniform vec4 bg;
in vec2 v_texcoord;
in vec3 f_normal;
// The texture.
uniform sampler2D u_texture;
uniform int grayscale;
uniform int text;

void main() {

    //coord is between -0.5 and 0.5
    //vec2 coord = gl_PointCoord - vec2(0.5,0.5);  
    vec4 o =texture(u_texture, v_texcoord);

    if(text==1){
        out_color=vec4(1.0,1.0,1.0,o.g);
    }else if (text==2){
        out_color = o ;
    }else{
        out_color = o ; 

        // because v_normal is a varying it's interpolated
        // so it will not be a unit vector. Normalizing it
        // will make it a unit vector again
        vec3 normal = normalize(f_normal);
      
        float light = dot(normal, normalize(vec3(-1.0,1.0,1.0)));
        light=min(1.0,light+0.9);
    
        // Lets multiply just the color portion (not the alpha)
        // by the light
        out_color.rgb *= light;
    }

    if(grayscale==1){
        // grayscale
        // https://stackoverflow.com/questions/31729326/glsl-grayscale-shader-removes-transparency
        float coll =  0.299 * out_color.r + 0.587 * out_color.g + 0.114 * out_color.b;
        out_color.r=coll;
        out_color.g=coll;
        out_color.b=coll;       
    }
}
"#;


const VERT_SHADER_STR: &str = r#"#version 300 es
in vec3 position;
in vec2 a_texcoord;
in vec3 v_normal;
uniform mat4 mmatrix;
uniform float point_size;
out vec3 f_normal;
out vec2 v_texcoord;
void main() {
    gl_PointSize = point_size;
    vec4 pp=vec4(position,1.0);
    vec4 j = mmatrix*pp;
    gl_Position = j;
    v_texcoord=a_texcoord;
    f_normal=v_normal;
}
"#;






pub type Vertex = [f32; 3];


pub struct TextureBuffer{
    pub(crate) texture: web_sys::WebGlTexture,
    pub(crate) ctx: WebGl2RenderingContext,
     width:i32,
    height:i32
}
impl Drop for TextureBuffer{
    fn drop(&mut self){
        self.ctx.delete_texture(Some(&self.texture));
    }
}
impl TextureBuffer{
    pub fn texture(&self)->&web_sys::WebGlTexture{
        &self.texture
    }
    pub fn width(&self)->i32{
        self.width
    }
    pub fn height(&self)->i32{
        self.height
    }
    pub fn new(ctx:&WebGl2RenderingContext)->TextureBuffer{
        let texture = ctx.create_texture().unwrap_throw();
        ctx.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        
        let width=1;
        let height=1;
        // Fill the texture with a 1x1 blue pixel.
        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            width, //width
            height, //height
            0, //border
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&[0, 0, 255, 255])).unwrap_throw();

        Self{
            ctx:ctx.clone(),
            texture,
            width,
            height
        }   
    }
    pub fn update(&mut self,width:usize,height:usize,image:&[u8]){
        //log!(format!("image bytes:{:?}",image.len()));
        self.ctx.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
        // self.ctx.compressed_tex_image_2d_with_u8_array(
        //     WebGl2RenderingContext::TEXTURE_2D,
        //     0,
        //     WebGl2RenderingContext::RGBA,
        //     width as i32,
        //     height as i32,
        //     0,
        //     image
        // );

        //log!("attemptying to make image!!!");
        // let arr=js_sys::Uint8ClampedArray::new_with_length(image.len() as u32);
        // arr.copy_from(image);

       
        //TODO leverage javascript to load png instead to avoid image dependancy??

        //let image = image::load_from_memory_with_format(&image, image::ImageFormat::Png).unwrap();
        //let rgba_image = image.to_rgba8();
        
        //https://stackoverflow.com/questions/70309403/updating-html-canvas-imagedata-using-rust-webassembly
        let clamped_buf: Clamped<&[u8]> = Clamped(image);
        let image = web_sys::ImageData::new_with_u8_clamped_array_and_sh(clamped_buf,width as u32,height as u32).map_err(|e|log!(e)).unwrap_throw();
        self.width=width as i32;
        self.height=height as i32;
        self.ctx.tex_image_2d_with_u32_and_u32_and_image_data(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            &image
        ).unwrap_throw();

        self.ctx.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::LINEAR as i32);
        self.ctx.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::LINEAR as i32);
        self.ctx.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);
        self.ctx.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);
        


        //log!("send111");

        // self.ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        //     WebGl2RenderingContext::TEXTURE_2D,
        //     0,
        //     WebGl2RenderingContext::RGBA as i32,
        //     width as i32, //width
        //     height as i32, //height
        //     0, //border
        //     WebGl2RenderingContext::RGBA,
        //     WebGl2RenderingContext::UNSIGNED_BYTE,
        //     Some(image)).unwrap_throw();
        
    }
}



pub struct GenericBuffer<T,L,J>(
    pub Buffer,
    std::marker::PhantomData<T>,
    L,
    J
);

impl<T:byte_slice_cast::ToByteSlice,L:BufferKind,J:BufferDyn> GenericBuffer<T,L,J>{
    pub fn new(ctx:&WebGl2RenderingContext)->Result<Self,String>
    {
        Ok(GenericBuffer(Buffer::new(ctx)?,std::marker::PhantomData,L::default(),J::default()))
    }

    pub fn update(&mut self, vertices: &[T])
    {
        // Now that the image has loaded make copy it to the texture.
        let ctx = &self.0.ctx;

        self.0.num_verts = vertices.len();

        ctx.bind_buffer(self.2.get(), Some(&self.0.buffer));

        use byte_slice_cast::*;

        let points_buf=vertices.as_byte_slice();

        ctx.buffer_data_with_u8_array(
            self.2.get(),
            points_buf,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }
}

pub type TextureCoordBuffer = GenericBuffer<[f32;2],ArrayKind,StaticKind>;
pub type Vert3Buffer = GenericBuffer<[f32;3],ArrayKind,StaticKind>;
pub type IndexBuffer=GenericBuffer<u16,ElementKind,StaticKind>;


pub trait BufferKind:Default{
    fn get(&self)->u32;
}

#[derive(Default)]
pub struct ElementKind;
impl BufferKind for ElementKind{
    fn get(&self)->u32{
        WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER
    }
}
#[derive(Default)]
pub struct ArrayKind;
impl BufferKind for ArrayKind{
    fn get(&self)->u32{
        WebGl2RenderingContext::ARRAY_BUFFER
    }
}

pub trait BufferDyn:Default{
    fn get(&self)->u32;
}

#[derive(Default)]
pub struct DynamicKind;
impl BufferDyn for DynamicKind{
    fn get(&self)->u32{
        WebGl2RenderingContext::DYNAMIC_DRAW
    }
}
#[derive(Default)]
pub struct StaticKind;
impl BufferDyn for StaticKind{
    fn get(&self)->u32{
        WebGl2RenderingContext::STATIC_DRAW
    }
}


struct Args<'a> {
    pub verts: &'a Buffer,
    pub indexes:Option<&'a IndexBuffer>,
    pub primitive: u32,
    pub texture:&'a TextureBuffer,
    pub texture_coords:&'a TextureCoordBuffer,
    pub grayscale:bool,
    pub matrix:&'a [f32;16],
    pub point_size: f32,
    pub normals:&'a Buffer,
    pub text:bool,
    pub lighting:bool
}


use wasm_bindgen::{prelude::*, Clamped};

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
        
        //https://webglfundamentals.org/webgl/lessons/webgl-text-texture.html
        self.pixel_storei(WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL,1);
        self.enable(WebGl2RenderingContext::BLEND);
        
        self.blend_func(
            //WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,    
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        self.enable(WebGl2RenderingContext::DEPTH_TEST);
        self.enable(WebGl2RenderingContext::CULL_FACE);
        
    }
    // pub fn buffer_dynamic(&self) -> DynamicBuffer {
    //     DynamicBuffer::new(self).unwrap_throw()
    // }

    // pub fn buffer_static_clear(&self, a: &mut Vec<Vertex>) -> StaticBuffer {
    //     let b = self.buffer_static_no_clear(a);
    //     a.clear();
    //     b
    // }

    // pub fn buffer_static_no_clear(&self, a: &[Vertex]) -> StaticBuffer {
    //     StaticBuffer::new(self, a).unwrap_throw()
    // }

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
            texture,
            texture_coords,
            matrix,
            indexes,
            point_size,
            normals,
            grayscale,
            text,
            lighting
            //world_inverse_transpose
        } = args;

        assert_eq!(verts.ctx, self.ctx);


        //if as_square {
            self.square_program
                .draw(texture,texture_coords,indexes,verts, primitive, &matrix, point_size,normals,grayscale,text,lighting);
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
    pub fn view<'a>(&'a mut self, matrix:&'a cgmath::Matrix4<f32>) -> View<'a> {
        self.view2(matrix.as_ref())
    }

    pub fn view2<'a>(&'a mut self, matrix:&'a [f32;16]) -> View<'a> {
        View {
            sys: self,
            matrix
        }
    }
}


pub trait Drawable{
    fn draw(&self, view: &mut View);
    fn draw_ext(
        &self,
        view: &mut View,
        grayscale: bool,
        text: bool,
        _linear: bool,
        lighting: bool,
    );
}
///
/// A view to draw in. See [`ShaderSystem::view`]
///
#[must_use]
pub struct View<'a> {
    sys: &'a mut ShaderSystem,
    matrix:&'a [f32;16],
    // world_inverse_transpose:&'a [f32;16],
    // offset: [f32; 2],
    // dim: [f32; 2],
}
impl View<'_> {
    pub fn draw_a_thing(&mut self,f:&impl Drawable){
        f.draw(self);
    }
    pub fn draw_a_thing_ext(&mut self,f:&impl Drawable,grayscale: bool,
        text: bool,
        _linear: bool,
        lighting: bool){
        f.draw_ext(self,grayscale,text,_linear,lighting);
    }
    // pub fn draw_squares(&mut self, verts: &Buffer, point_size: f32, color: &[f32; 4]) {
    //     self.sys.draw(Args {
    //         verts,
    //         primitive: WebGl2RenderingContext::POINTS,
    //         matrix:self.matrix,
    //         color,
    //         point_size,
    //     })
    // }
    pub fn draw(&mut self,primitive:u32,texture:&TextureBuffer,texture_coords:&TextureCoordBuffer, verts: &Buffer,indexes:Option<&IndexBuffer>,normals:&Buffer,grayscale:bool,text:bool,linear:bool,lighting:bool) {
        self.sys.draw(Args {
            texture,
            texture_coords,
            verts,
            primitive,
            matrix: self.matrix,
            indexes,
            normals,
            // world_inverse_transpose:self.world_inverse_transpose,
            point_size: 1.0,
            grayscale,
            text,
            lighting
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
