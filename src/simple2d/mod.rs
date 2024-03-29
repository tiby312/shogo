//!
//! A simple webgl 2d drawing system that draws shapes using many small circles or squares.
//!
//! The data sent to the gpu is minimized by only sending the positions of the vertex.
//! The color can also be changed for all vertices in a buffer.
//!
use gloo::console::log;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};
pub mod shader;

use shader::*;
use WebGl2RenderingContext as GL;

pub type Vertex = [f32; 3];

pub struct TextureBuffer {
    pub(crate) texture: web_sys::WebGlTexture,
    pub(crate) ctx: WebGl2RenderingContext,
    width: i32,
    height: i32,
}
impl Drop for TextureBuffer {
    fn drop(&mut self) {
        self.ctx.delete_texture(Some(&self.texture));
    }
}
impl TextureBuffer {
    pub fn texture(&self) -> &web_sys::WebGlTexture {
        &self.texture
    }
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn bind(&self, ctx: &WebGl2RenderingContext) {
        ctx.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
    }

    pub fn new(ctx: &WebGl2RenderingContext) -> TextureBuffer {
        let texture = ctx.create_texture().unwrap_throw();
        ctx.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        let width = 1;
        let height = 1;
        // Fill the texture with a 1x1 blue pixel.
        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            width,  //width
            height, //height
            0,      //border
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&[0, 0, 255, 255]),
        )
        .unwrap_throw();

        Self {
            ctx: ctx.clone(),
            texture,
            width,
            height,
        }
    }
    pub fn update(&mut self, width: usize, height: usize, image: &[u8]) {
        //log!(format!("image bytes:{:?}",image.len()));
        self.ctx
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
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
        let image = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            clamped_buf,
            width as u32,
            height as u32,
        )
        .map_err(|e| log!(e))
        .unwrap_throw();
        self.width = width as i32;
        self.height = height as i32;
        self.ctx
            .tex_image_2d_with_u32_and_u32_and_image_data(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                &image,
            )
            .unwrap_throw();

        self.ctx.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        self.ctx.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        self.ctx.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        self.ctx.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );

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

// //TODO remove this. not really needed when using VAO
// pub struct GenericBuffer<T, L, J> {
//     buffer: web_sys::WebGlBuffer,
//     num_verts: usize,
//     ctx: WebGl2RenderingContext,
//     _p: std::marker::PhantomData<T>,
//     kind: L,
//     dynamic: J,
// }

// impl<
//         T: byte_slice_cast::ToByteSlice + NumComponent + ComponentType,
//         L: BufferKind,
//         J: BufferDyn,
//     > GenericBuffer<T, L, J>
// {
//     pub fn new(ctx: &WebGl2RenderingContext) -> Result<Self, String> {
//         let buffer = ctx.create_buffer().ok_or("failed to create buffer")?;

//         Ok(GenericBuffer {
//             buffer,
//             _p: std::marker::PhantomData,
//             kind: L::default(),
//             dynamic: J::default(),
//             num_verts: 0,
//             ctx: ctx.clone(),
//         })
//     }

//     pub fn setup_attrib<K: ProgramAttrib<NumComponent = T>>(
//         &self,
//         att: K,
//         ctx: &WebGl2RenderingContext,
//         prog: &GlProgram,
//     ) {
//         ctx.vertex_attrib_pointer_with_i32(
//             att.get_attrib(prog) as u32,
//             T::num(),
//             T::component_type(),
//             false,
//             0,
//             0,
//         );
//     }

//     // //TODO use
//     // pub fn attrib_divisor_of_one<K: ProgramAttrib<NumComponent = T>>(
//     //     &self,
//     //     att: K,
//     //     ctx: &WebGl2RenderingContext,
//     //     prog: &GlProgram,
//     // ) {
//     //     ctx.vertex_attrib_divisor(att.get_attrib(prog) as u32, 1)
//     // }

//     pub fn bind(&self, ctx: &WebGl2RenderingContext) {
//         ctx.bind_buffer(self.kind.get(), Some(&self.buffer));
//     }

//     pub fn update(&mut self, vertices: &[T]) {
//         // Now that the image has loaded make copy it to the texture.
//         let ctx = &self.ctx;

//         self.num_verts = vertices.len();

//         ctx.bind_buffer(self.kind.get(), Some(&self.buffer));

//         use byte_slice_cast::*;

//         let points_buf = vertices.as_byte_slice();

//         ctx.buffer_data_with_u8_array(self.kind.get(), points_buf, self.dynamic.get());
//     }
// }

pub trait NumComponent {
    fn num() -> i32;
}
impl NumComponent for [f32; 2] {
    fn num() -> i32 {
        2
    }
}
impl NumComponent for [f32; 3] {
    fn num() -> i32 {
        3
    }
}

impl NumComponent for [f32; 16] {
    fn num() -> i32 {
        16
    }
}
impl NumComponent for u16 {
    fn num() -> i32 {
        1
    }
}

pub struct Mat4Buffer {
    buffer: WebGlBuffer,
    num_verts: usize,
    ctx: GL,
}
impl Mat4Buffer {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<Self, String> {
        let buffer = ctx.create_buffer().ok_or("failed to create buffer")?;

        Ok(Mat4Buffer {
            buffer,
            num_verts: 0,
            ctx: ctx.clone(),
        })
    }
    pub fn bind(&self, ctx: &WebGl2RenderingContext) {
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&self.buffer));
    }
    pub fn update(&mut self, vertices: &[[f32; 16]]) {
        // Now that the image has loaded make copy it to the texture.
        let ctx = &self.ctx;

        self.num_verts = vertices.len();

        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&self.buffer));

        use byte_slice_cast::*;

        let points_buf = vertices.as_byte_slice();

        ctx.buffer_data_with_u8_array(GL::ARRAY_BUFFER, points_buf, GL::DYNAMIC_DRAW);
    }
}

//TODO use this
//pub type Mat4Buffer = GenericBuffer<[f32; 16], ArrayKind, DynamicKind>;

// pub type TextureCoordBuffer = GenericBuffer<[f32; 2], ArrayKind, StaticKind>;
// pub type Vert3Buffer = GenericBuffer<[f32; 3], ArrayKind, StaticKind>;
// pub type IndexBuffer = GenericBuffer<u16, ElementKind, StaticKind>;

// pub trait BufferKind: Default {
//     fn get(&self) -> u32;
// }

// #[derive(Default)]
// pub struct ElementKind;
// impl BufferKind for ElementKind {
//     fn get(&self) -> u32 {
//         WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER
//     }
// }
// #[derive(Default)]
// pub struct ArrayKind;
// impl BufferKind for ArrayKind {
//     fn get(&self) -> u32 {
//         WebGl2RenderingContext::ARRAY_BUFFER
//     }
// }

// pub trait BufferDyn: Default {
//     fn get(&self) -> u32;
// }

// #[derive(Default)]
// pub struct DynamicKind;
// impl BufferDyn for DynamicKind {
//     fn get(&self) -> u32 {
//         WebGl2RenderingContext::DYNAMIC_DRAW
//     }
// }
// #[derive(Default)]
// pub struct StaticKind;
// impl BufferDyn for StaticKind {
//     fn get(&self) -> u32 {
//         WebGl2RenderingContext::STATIC_DRAW
//     }
// }

struct Args<'a> {
    pub res: &'a VaoResult,
    pub primitive: u32,
    pub texture: &'a TextureBuffer,
    pub grayscale: bool,
    pub matrix: &'a [[f32; 16]],
    pub point_size: f32,
    pub text: bool,
    pub lighting: bool,
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

    ///
    /// Sets up alpha blending and disables depth testing.
    ///
    pub fn setup_alpha(&self) {
        //https://webglfundamentals.org/webgl/lessons/webgl-text-texture.html
        self.pixel_storei(WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 1);
        self.enable(WebGl2RenderingContext::BLEND);

        self.blend_func(
            //WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        self.enable(WebGl2RenderingContext::DEPTH_TEST);
        self.enable(WebGl2RenderingContext::CULL_FACE);
    }
    

    pub fn shader_system(&self) -> ShaderSystem {
        ShaderSystem::new(self)
            .map_err(|e| {
                log!(format!("{:?}", e));
                e
            })
            .unwrap_throw()
    }


    pub fn draw_clear(&self, color: [f32; 4]) {
        let [a, b, c, d] = color;
        self.ctx.clear_color(a, b, c, d);
        self.ctx.clear(
            web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT
                | web_sys::WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );
    }
}


///
/// A simple shader program that allows the user to draw simple primitives.
///
pub struct ShaderSystem {
    pub square_program: GlProgram,
    pub ctx: WebGl2RenderingContext,
}

impl Drop for ShaderSystem {
    fn drop(&mut self) {
        self.ctx.delete_program(Some(&self.square_program.program));
    }
}

impl ShaderSystem {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<ShaderSystem, String> {
        let square_program = GlProgram::new(ctx)?;

        Ok(ShaderSystem {
            square_program,
            ctx: ctx.clone(),
        })
    }

    pub fn view2<'a>(&'a mut self, matrix: &'a [[f32; 16]]) -> View<'a> {
        View { sys: self, matrix }
    }
}

pub trait Drawable {
    fn draw(&self, view: &mut View);
    fn draw_ext(&self, view: &mut View, grayscale: bool, text: bool, _linear: bool, lighting: bool);
}
///
/// A view to draw in. See [`ShaderSystem::view`]
///
#[must_use]
pub struct View<'a> {
    sys: &'a mut ShaderSystem,
    matrix: &'a [[f32; 16]],
}
impl View<'_> {
    pub fn draw_a_thing(&mut self, f: &impl Drawable) {
        f.draw(self);
    }
    pub fn draw_a_thing_ext(
        &mut self,
        f: &impl Drawable,
        grayscale: bool,
        text: bool,
        _linear: bool,
        lighting: bool,
    ) {
        f.draw_ext(self, grayscale, text, _linear, lighting);
    }

    pub fn draw(
        &mut self,
        primitive: u32,
        texture: &TextureBuffer,
        res: &VaoResult,
        grayscale: bool,
        text: bool,
        linear: bool,
        lighting: bool,
    ) {
        self.sys.square_program.draw(shader::Argss {
            texture,
            primitive,
            mmatrix: self.matrix,
            res,
            // world_inverse_transpose:self.world_inverse_transpose,
            point_size: 1.0,
            grayscale,
            text,
            lighting,
        })
    }

}



//TODO why is this here?
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
