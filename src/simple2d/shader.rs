use web_sys::WebGlShader;
use web_sys::WebGlUniformLocation;
use web_sys::{WebGl2RenderingContext, WebGlProgram};

use super::IndexBuffer;
use super::TextureBuffer;
use super::TextureCoordBuffer;
use super::Vert3Buffer;
use super::*;

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
in mat4 mmatrix;
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

pub struct Argss<'a> {
    pub texture: &'a TextureBuffer,
    pub texture_coords: &'a TextureCoordBuffer,
    pub indexes: Option<&'a IndexBuffer>,
    pub position: &'a Vert3Buffer,
    pub normals: &'a Vert3Buffer,
    pub primitive: u32,
    pub mmatrix: &'a [[f32; 16]],
    pub point_size: f32,
    pub grayscale: bool,
    pub text: bool,
    pub lighting: bool,
}

impl GlProgram {
    pub fn draw(&mut self, argss: Argss) {
        let Argss {
            texture,
            texture_coords,
            indexes,
            position,
            normals,
            primitive,
            mmatrix,
            point_size,
            grayscale,
            text,
            lighting,
        } = argss;
        if position.num_verts == 0 {
            return;
        }

        let context = &position.ctx;

        context.use_program(Some(&self.program));

        self.matrix_buffer.update(mmatrix);
        self.matrix_buffer.bind(context);
        self.matrix_buffer.setup_attrib_special(context, self);
        // self.matrix_buffer.setup_attrib(MMatrix,context,self);
        // self.matrix_buffer.attrib_divisor_of_one(MMatrix, context, self);

        //context.uniform_matrix4fv_with_f32_array(Some(&self.mmatrix), false, mmatrix);

        let kk: i32 = if grayscale { 1 } else { 0 };
        context.uniform1i(Some(&self.grayscale), kk);

        let kk: i32 = if text {
            1
        } else if !lighting {
            2
        } else {
            0
        };

        context.uniform1i(Some(&self.text), kk);
        context.uniform1f(Some(&self.point_size), point_size);

        texture_coords.bind(context);
        texture_coords.setup_attrib(TexCoord, context, self);

        position.bind(context);
        position.setup_attrib(Position3, context, self);

        normals.bind(context);
        normals.setup_attrib(Normal, context, self);

        texture.bind(context);

        if let Some(indexes) = indexes {
            indexes.bind(context);
            //context.draw_elements_with_i32(primitive, indexes.num_verts as i32,WebGl2RenderingContext::UNSIGNED_SHORT,0);
            let instance_count = mmatrix.len() as i32;
            context.draw_elements_instanced_with_i32(
                primitive,
                indexes.num_verts as i32,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0,
                instance_count,
            )
        } else {
            context.draw_arrays(primitive, 0, position.num_verts as i32)
        }
    }

    pub fn new(context: &WebGl2RenderingContext) -> Result<Self, String> {
        let vs = VERT_SHADER_STR;
        let fs = SQUARE_FRAG_SHADER_STR;

        let vert_shader = compile_shader(context, WebGl2RenderingContext::VERTEX_SHADER, vs)?;
        let frag_shader = compile_shader(context, WebGl2RenderingContext::FRAGMENT_SHADER, fs)?;
        let program = link_program(context, &vert_shader, &frag_shader)?;

        context.delete_shader(Some(&vert_shader));
        context.delete_shader(Some(&frag_shader));

        let grayscale = context
            .get_uniform_location(&program, "grayscale")
            .ok_or_else(|| "uniform err".to_string())?;

        let text = context
            .get_uniform_location(&program, "text")
            .ok_or_else(|| "uniform err".to_string())?;

        // let mmatrix = context
        //     .get_uniform_location(&program, "mmatrix")
        //     .ok_or_else(|| "uniform err".to_string())?;

        let point_size = context
            .get_uniform_location(&program, "point_size")
            .ok_or_else(|| "uniform err".to_string())?;

        let mmatrix = context.get_attrib_location(&program, "mmatrix");

        let position = context.get_attrib_location(&program, "position");

        let normal = context.get_attrib_location(&program, "v_normal");

        let texcoord = context.get_attrib_location(&program, "a_texcoord");

        if mmatrix < 0 {
            return Err("attribute err".to_string());
        }

        let position = position as u32;
        let normal = normal as u32;
        let texcoord = texcoord as u32;
        let mmatrix = mmatrix as u32;
        //context.enable_vertex_attrib_array(mmatrix);
        for i in 0..4 {
            let loc = mmatrix + i;
            context.enable_vertex_attrib_array(loc);
        }

        context.enable_vertex_attrib_array(texcoord);

        context.enable_vertex_attrib_array(position);

        context.enable_vertex_attrib_array(normal);

        Ok(GlProgram {
            program,
            mmatrix,
            point_size,
            normal,
            position,
            texcoord,
            grayscale,
            text,
            matrix_buffer: Mat4Buffer::new(context).unwrap(),
        })
    }
}

impl Mat4Buffer {
    pub fn setup_attrib_special(&self, ctx: &WebGl2RenderingContext, program: &GlProgram) {
        let bytesPerMatrix = 4 * 16;
        let matrixLoc = program.mmatrix;

        for i in 0..4 {
            let loc = matrixLoc + i;

            let offset = (i * 16) as i32;
            // note the stride and offset

            ctx.vertex_attrib_pointer_with_i32(
                loc as u32,
                4,
                WebGl2RenderingContext::FLOAT,
                false,
                bytesPerMatrix,
                offset,
            );

            ctx.vertex_attrib_divisor(loc as u32, 1);
        }
    }
}

struct Position3;
struct TexCoord;
struct Normal;

struct MMatrix;

pub trait ProgramAttrib {
    type NumComponent;
    fn get_attrib(&self, a: &GlProgram) -> u32;
}
impl ProgramAttrib for MMatrix {
    type NumComponent = [f32; 16];

    fn get_attrib(&self, a: &GlProgram) -> u32 {
        a.mmatrix
    }
}

impl ProgramAttrib for Position3 {
    type NumComponent = [f32; 3];

    fn get_attrib(&self, a: &GlProgram) -> u32 {
        a.position
    }
}
impl ProgramAttrib for TexCoord {
    type NumComponent = [f32; 2];

    fn get_attrib(&self, a: &GlProgram) -> u32 {
        a.texcoord
    }
}
impl ProgramAttrib for Normal {
    type NumComponent = [f32; 3];

    fn get_attrib(&self, a: &GlProgram) -> u32 {
        a.normal
    }
}

pub struct GlProgram {
    pub(crate) program: WebGlProgram,
    mmatrix: u32,
    point_size: WebGlUniformLocation,
    grayscale: WebGlUniformLocation,
    position: u32,
    texcoord: u32,
    normal: u32,
    text: WebGlUniformLocation,
    matrix_buffer: Mat4Buffer,
}

fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

pub struct AttrMMatrix;
pub struct AttrMMatrixLoaded;
impl AttrMMatrix {
    pub fn load(&self, arr: ArrBound<[f32; 16]>, program: &GlProgram) -> AttrMMatrixLoaded {
        let ctx = &arr.buffer.ctx;
        let bytesPerMatrix = 4 * 16;
        let matrixLoc = program.mmatrix;

        for i in 0..4 {
            let loc = matrixLoc + i;

            let offset = (i * 16) as i32;
            // note the stride and offset

            ctx.vertex_attrib_pointer_with_i32(
                loc as u32,
                4,
                WebGl2RenderingContext::FLOAT,
                false,
                bytesPerMatrix,
                offset,
            );

            ctx.vertex_attrib_divisor(loc as u32, 1);
        }
        AttrMMatrixLoaded
    }
}

pub struct AttrPosition;
#[must_use]
pub struct AttrPositionLoaded;
impl AttrPosition {
    pub fn load(&self, arr: ArrBound<[f32; 2]>, prog: &GlProgram) -> AttrPositionLoaded {
        arr.buffer.ctx.vertex_attrib_pointer_with_i32(
            prog.position as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        AttrPositionLoaded
    }
}
pub struct AttrNormal;
pub struct AttrNormalLoaded;

impl AttrNormal {
    pub fn load(&self, arr: ArrBound<[f32; 2]>, prog: &GlProgram) -> AttrNormalLoaded {
        arr.buffer.ctx.vertex_attrib_pointer_with_i32(
            prog.normal as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        AttrNormalLoaded
    }
}

pub struct AttrTextureCoord;
pub struct AttrTextureCoordLoaded;
impl AttrTextureCoord {
    pub fn load(&self, arr: ArrBound<[f32; 2]>, prog: &GlProgram) -> AttrTextureCoordLoaded {
        arr.buffer.ctx.vertex_attrib_pointer_with_i32(
            prog.normal as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        AttrTextureCoordLoaded
    }
}

fn foo(
    prog: &GlProgram,
    b: &mut BindStuff,
    texture_coord_buffer: &mut Buff<[f32; 2]>,
    index_buffer: &mut Buff<u16>,
    tex_buffer: &mut TextureBuffer,
    position_buffer: &mut Buff<[f32; 2]>,
    normal_buffer: &mut Buff<[f32; 2]>,
    mmatrixes: &[[f32; 16]],
    matrix_buff: &mut Buff<[f32; 16]>,
) {
    let ctx=&index_buffer.ctx.clone();

    let mut bound_matrix = b.arr.bind(matrix_buff);
    bound_matrix.update(mmatrixes);
    let num_instances=mmatrixes.len();

    let matrixes = AttrMMatrix.load(bound_matrix, prog);

    let bound_positions = b.arr.bind(position_buffer);
    let positions = AttrPosition.load(bound_positions, prog);

    let bound_normals = b.arr.bind(normal_buffer);
    let normals = AttrNormal.load(bound_normals, prog);

    let bound_texcoord = b.arr.bind(texture_coord_buffer);
    let texcoord = AttrTextureCoord.load(bound_texcoord, prog);

    let texture = b.tex.bind(tex_buffer);

    let indexes = b.elem.bind(index_buffer);

    Finish {
        positions,
        normals,
        texcoord,
        matrixes,
        indexes,
        texture,
        num_instances
    }.draw(ctx);
}

struct Finish<'a>{
    positions:AttrPositionLoaded,
    normals:AttrNormalLoaded,
    texcoord:AttrTextureCoordLoaded,
    matrixes:AttrMMatrixLoaded,
    indexes:ElemBound<'a,u16>,
    texture:TexBound<'a>,
    num_instances:usize
}
impl<'a> Finish<'a>{
    fn draw(&self,ctx:&WebGl2RenderingContext){
        //context.draw_elements_with_i32(primitive, indexes.num_verts as i32,WebGl2RenderingContext::UNSIGNED_SHORT,0);
        let instance_count = self.num_instances as i32;
        ctx.draw_elements_instanced_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            self.indexes.buffer.num_verts as i32,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
            instance_count,
        )
    }
}

pub struct BindStuff {
    arr: ArrBindPoint,
    elem: ElemBindPoint,
    tex: TexBindPoint,
}

pub struct Buff<T> {
    ctx: WebGl2RenderingContext,
    id: web_sys::WebGlBuffer,
    _p: std::marker::PhantomData<[T]>,
    num_verts: usize,
}

pub struct ArrBound<'a, T> {
    a: &'a mut ArrBindPoint,
    buffer: &'a mut Buff<T>,
}
impl<'a, T: byte_slice_cast::ToByteSlice> ArrBound<'a, T> {
    fn update(&mut self, update: &[T]) {
        // Now that the image has loaded make copy it to the texture.
        let ctx = &self.buffer.ctx;

        self.buffer.num_verts = update.len();

        use byte_slice_cast::*;

        let points_buf = update.as_byte_slice();
        //TODO allow user to specify static/dynamic
        ctx.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            points_buf,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }
}

pub struct ElemBound<'a, T> {
    a: &'a mut ElemBindPoint,
    buffer: &'a mut Buff<T>,
}
pub struct TexBound<'a> {
    a: &'a mut TexBindPoint,
    buffer: &'a mut TextureBuffer,
}
pub struct ElemBindPoint;
pub struct TexBindPoint;
pub struct ArrBindPoint;

impl ArrBindPoint {
    fn bind<'b,T>(&'b mut self, buffer: &'b mut Buff<T>) -> ArrBound<'b, T> {
        buffer
            .ctx
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer.id));
        ArrBound { a: self, buffer }
    }
}

impl ElemBindPoint {
    fn bind<'b,T>(&'b mut self, buffer: &'b mut Buff<T>) -> ElemBound<'b, T> {
        buffer.ctx.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&buffer.id),
        );
        ElemBound { a: self, buffer }
    }
}

impl TexBindPoint {
    fn bind<'b>(&'b mut self, buffer: &'b mut TextureBuffer) -> TexBound<'b> {
        buffer
            .ctx
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&buffer.texture));

        TexBound { a: self, buffer }
    }
}
