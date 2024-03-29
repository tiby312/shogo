use web_sys::WebGlBuffer;
use web_sys::WebGlShader;
use web_sys::WebGlUniformLocation;
use web_sys::{WebGl2RenderingContext, WebGlProgram};

use WebGl2RenderingContext as GL;

use super::TextureBuffer;

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
    pub res: &'a VaoResult,
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
            res,
            mmatrix,
            point_size,
            grayscale,
            text,
            lighting,
        } = argss;

        let context = &self.ctx;

        context.use_program(Some(&self.program));

        self.matrix_buffer.update(mmatrix);

        texture.bind(context);

        context.bind_vertex_array(Some(&res.vao));

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

        context.draw_elements_instanced_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            res.num_index as i32,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
            mmatrix.len() as i32,
        );
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

        Ok(GlProgram {
            ctx: context.clone(),
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


//TODO destroy buffers in destructor
pub struct VaoResult {
    num_index: usize,
    num_vertex: usize,
    index: WebGlBuffer,
    tex_coord: WebGlBuffer,
    position: WebGlBuffer,
    normal: WebGlBuffer,
    vao: web_sys::WebGlVertexArrayObject,
}

pub fn create_vao2(
    ctx: &WebGl2RenderingContext,
    program: &GlProgram,
    tex_coords: &[[f32; 2]],
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    indices: &[u16],
    mat: &Mat4Buffer,
) -> VaoResult {
    use byte_slice_cast::*;

    let vao = ctx.create_vertex_array().unwrap();
    ctx.bind_vertex_array(Some(&vao));

    ctx.enable_vertex_attrib_array(program.texcoord);
    ctx.enable_vertex_attrib_array(program.position);
    ctx.enable_vertex_attrib_array(program.normal);
    for i in 0..4 {
        let loc = program.mmatrix + i;
        ctx.enable_vertex_attrib_array(loc);
    }

    let tex_coord = ctx.create_buffer().unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&tex_coord));
    ctx.buffer_data_with_u8_array(
        GL::ARRAY_BUFFER,
        tex_coords.as_byte_slice(),
        GL::STATIC_DRAW,
    );
    ctx.vertex_attrib_pointer_with_i32(
        program.texcoord as u32,
        <[f32; 2]>::num(),
        GL::FLOAT,
        false,
        0,
        0,
    );

    let position = ctx.create_buffer().unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&position));
    ctx.buffer_data_with_u8_array(GL::ARRAY_BUFFER, positions.as_byte_slice(), GL::STATIC_DRAW);
    ctx.vertex_attrib_pointer_with_i32(
        program.position as u32,
        <[f32; 3]>::num(),
        GL::FLOAT,
        false,
        0,
        0,
    );

    let normal = ctx.create_buffer().unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&normal));
    ctx.buffer_data_with_u8_array(GL::ARRAY_BUFFER, normals.as_byte_slice(), GL::STATIC_DRAW);
    ctx.vertex_attrib_pointer_with_i32(
        program.normal as u32,
        <[f32; 3]>::num(),
        GL::FLOAT,
        false,
        0,
        0,
    );

    mat.bind(ctx);
    mat.setup_attrib_special(ctx, program);

    let index = ctx.create_buffer().unwrap();
    ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index));
    ctx.buffer_data_with_u8_array(
        GL::ELEMENT_ARRAY_BUFFER,
        indices.as_byte_slice(),
        GL::STATIC_DRAW,
    );

    ctx.bind_vertex_array(None);

    VaoResult {
        num_index: indices.len(),
        num_vertex: positions.len(),
        index,
        tex_coord,
        position,
        normal,
        vao,
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
    pub matrix_buffer: Mat4Buffer,
    pub ctx: WebGl2RenderingContext,
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




trait NumComponent {
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