use web_sys::{WebGlProgram, WebGl2RenderingContext, WebGlShader};

use crate::circle_program::*;


const SQUARE_FRAG_SHADER_STR:&'static str=r#"#version 300 es
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

const CIRCLE_FRAG_SHADER_STR:&'static str=r#"#version 300 es
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

const VERT_SHADER_STR:&'static str=r#"#version 300 es
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


#[repr(transparent)]
#[derive(Debug)]
pub struct Vertex(pub [f32;3]);



pub struct Args<'a>{
    pub ctx:&'a WebGl2RenderingContext,
    pub verts:&'a [Vertex],
    pub game_dim:[f32;2],
    pub as_square:bool,
    pub color:&'a [f32;4],
    pub offset:&'a [f32;2],
    pub point_size:f32
}


pub struct Foop<F>{
    func:F
}

impl<F:FnMut(Args)->Result<(),String>> Foop<F>{
    pub fn draw(&mut self,args:Args)->Result<(),String>{
        (self.func)(args)
    }
}

pub fn create_draw_system(ctx:&WebGl2RenderingContext)->Result<Foop<impl FnMut(Args)->Result<(),String>>,String>{
    
    let buffer = ctx.create_buffer().ok_or("failed to create buffer")?;

    let circle_program=CircleProgram::new(&ctx,VERT_SHADER_STR,CIRCLE_FRAG_SHADER_STR)?;
    let square_program=CircleProgram::new(&ctx,VERT_SHADER_STR,SQUARE_FRAG_SHADER_STR)?;
    Ok(Foop{func:move |Args{ctx,verts,game_dim,as_square,color,offset,point_size}:Args|{

        let scalex = 2.0 / game_dim[0];
        let scaley = 2.0 / game_dim[1];
        let tx = -1.0;
        let ty = 1.0;
        let matrix = [scalex, 0.0, 0.0, 0.0, -scaley, 0.0, tx, ty, 1.0];
        
        if as_square{
            square_program.draw(ctx,&buffer,*offset,&matrix,point_size,color,verts);
        }else{
            circle_program.draw(ctx,&buffer,*offset,&matrix,point_size,color,verts);
        };
    
        Ok(())
    }})
}


pub fn line(buffer:&mut Vec<Vertex>,radius:f32,start:[f32;2],end:[f32;2]){
    let offsetx=end[0]-start[0];
    let offsety=end[1]-start[1];

    let dis_sqr=offsetx*offsetx+offsety*offsety;
    let dis=dis_sqr.sqrt();

    let normx=offsetx/dis;
    let normy=offsety/dis;

    let num=(dis/(radius)).floor() as usize;

    for i in 0..num{
        let x=start[0]+(i as f32)*normx*radius;
        let y=start[1]+(i as f32)*normy*radius;
        buffer.push(Vertex([x,y,0.0]));
    }

}





pub fn compile_shader(
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

pub fn link_program(
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




