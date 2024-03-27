use web_sys::WebGlShader;
use web_sys::WebGlUniformLocation;
use web_sys::{WebGl2RenderingContext, WebGlProgram};

use super::BufferDyn;
use super::BufferKind;
use super::IndexBuffer;
use super::TextureBuffer;
use super::TextureCoordBuffer;
use super::Vert3Buffer;
use super::*;

impl GlProgram {
    pub fn draw(
        &self,
        texture:&TextureBuffer,
        texture_coords:&TextureCoordBuffer,
        indexes:Option<&IndexBuffer>,
        position: &Vert3Buffer,
        primitive: u32,
        mmatrix: &[f32; 16],
        point_size: f32,
        normals:&Vert3Buffer,
        grayscale:bool,
        text:bool,
        lighting:bool
        //world_inverse_transpose:&[f32;16]
    ) {
        if position.num_verts == 0 {
            return;
        }

        let context = &position.ctx;

        context.use_program(Some(&self.program));


        context.uniform_matrix4fv_with_f32_array(Some(&self.mmatrix), false, mmatrix);


        let kk:i32=if grayscale{1}else{0};
        context.uniform1i(Some(&self.grayscale), kk);

        let kk:i32=if text  {
            1
        }else if !lighting {
            2
        }else{
            0
        };

        context.uniform1i(Some(&self.text), kk);
        context.uniform1f(Some(&self.point_size), point_size);

        texture_coords.bind(context);
        texture_coords.setup_attrib(TexCoord,context,self);

        position.bind(context);
        position.setup_attrib(Position3,context,self);
        
        normals.bind(context);
        normals.setup_attrib(Normal,context,self);

        texture.bind(context);       
    
        if let Some(indexes)=indexes{
            indexes.bind(context);
            context.draw_elements_with_i32(primitive, indexes.num_verts as i32,WebGl2RenderingContext::UNSIGNED_SHORT,0)
        }else{
            context.draw_arrays(primitive, 0, position.num_verts as i32)
        }
    }

    pub fn new(context: &WebGl2RenderingContext, vs: &str, fs: &str) -> Result<Self, String> {
        let vert_shader = compile_shader(context, WebGl2RenderingContext::VERTEX_SHADER, vs)?;
        let frag_shader = compile_shader(context, WebGl2RenderingContext::FRAGMENT_SHADER, fs)?;
        let program = link_program(context, &vert_shader, &frag_shader)?;

        context.delete_shader(Some(&vert_shader));
        context.delete_shader(Some(&frag_shader));

        let grayscale=context.get_uniform_location(&program, "grayscale")
        .ok_or_else(|| "uniform err".to_string())?;
        
        let text=context.get_uniform_location(&program, "text")
        .ok_or_else(|| "uniform err".to_string())?;
        
        let mmatrix = context
            .get_uniform_location(&program, "mmatrix")
            .ok_or_else(|| "uniform err".to_string())?;


        let point_size = context
            .get_uniform_location(&program, "point_size")
            .ok_or_else(|| "uniform err".to_string())?;

        let position = context.get_attrib_location(&program, "position");

        let normal = context.get_attrib_location(&program, "v_normal");


        let texcoord = context.get_attrib_location(&program, "a_texcoord");

        if position < 0 {
            return Err("attribute err".to_string());
        }


        let position = position as u32;
        let normal=normal as u32;
        let texcoord=texcoord as u32;

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
            text
        })
    }
}


pub struct Position3;
pub struct TexCoord;
pub struct Normal;

pub trait ProgramAttrib{
    type NumComponent;
    fn get_attrib(&self,a:&GlProgram)->u32;
}
impl ProgramAttrib for Position3{
    type NumComponent=[f32;3];

    fn get_attrib(&self,a:&GlProgram)->u32{
        a.position
    }
}
impl ProgramAttrib for TexCoord{
    type NumComponent=[f32;2];
    
    fn get_attrib(&self,a:&GlProgram)->u32{
        a.texcoord
    }
}
impl ProgramAttrib for Normal{
    type NumComponent=[f32;3];

    fn get_attrib(&self,a:&GlProgram)->u32{
        a.normal
    }
}




// pub struct CurrentContext<A,E,T>{
//     array:A,
//     element:E,
//     texture:T
// }


// impl<A,E,T> CurrentContext<A,E,T>{
//     pub fn bind_array<A2>(self,array2:A2)->CurrentContext<A2,E,T>{
//         CurrentContext{array:array2,element:self.element,texture:self.texture}
//     }
// }


pub struct GlProgram {
    pub(crate) program: WebGlProgram,
    mmatrix: WebGlUniformLocation,
    point_size: WebGlUniformLocation,
    grayscale:WebGlUniformLocation,
    //world_inverse_transpose:WebGlUniformLocation,
    //bg: WebGlUniformLocation,
    position: u32,
    texcoord:u32,
    normal:u32,
    text:WebGlUniformLocation
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
