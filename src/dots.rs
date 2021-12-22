use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

use crate::circle_program::*;

pub use crate::circle_program::Vertex;

const SQUARE_FRAG_SHADER_STR: &'static str = r#"#version 300 es
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

const CIRCLE_FRAG_SHADER_STR: &'static str = r#"#version 300 es
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

const VERT_SHADER_STR: &'static str = r#"#version 300 es
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

pub struct DynamicBuffer(Buffer);

impl std::ops::Deref for DynamicBuffer {
    type Target = Buffer;
    fn deref(&self) -> &Buffer {
        &self.0
    }
}

impl DynamicBuffer {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<Self, String> {
        let buffer = ctx.create_buffer().ok_or("failed to create buffer")?;
        Ok(DynamicBuffer(Buffer {
            buffer,
            num_verticies: 0,
        }))
    }
    pub fn update(&mut self, context: &WebGl2RenderingContext, vertices: &[Vertex]) {
        self.0.num_verticies = vertices.len();

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.0.buffer));

        let n_bytes = vertices.len() * std::mem::size_of::<Vertex>();
        let points_buf: &[u8] =
            unsafe { std::slice::from_raw_parts(vertices.as_ptr() as *const u8, n_bytes) };

        context.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            points_buf,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }
}

pub struct Args<'a> {
    pub verts: &'a Buffer,
    pub game_dim: [f32; 2],
    pub as_square: bool,
    pub color: &'a [f32; 4],
    pub offset: &'a [f32; 2],
    pub point_size: f32,
}

pub struct ShaderSystem{
    circle_program:CircleProgram,
    square_program:CircleProgram,
    ctx:WebGl2RenderingContext
}

impl ShaderSystem{
    pub fn new(ctx: &WebGl2RenderingContext)->Result<ShaderSystem,String>{
        let circle_program = CircleProgram::new(&ctx, VERT_SHADER_STR, CIRCLE_FRAG_SHADER_STR)?;
        let square_program = CircleProgram::new(&ctx, VERT_SHADER_STR, SQUARE_FRAG_SHADER_STR)?;
    
        Ok(ShaderSystem{
            circle_program,
            square_program,
            ctx:ctx.clone()
        })
    }
    fn draw(&mut self,args:Args){
        let Args {
            verts,
            game_dim,
            as_square,
            color,
            offset,
            point_size,
        }=args;

        let scalex = 2.0 / game_dim[0];
        let scaley = 2.0 / game_dim[1];
        let tx = -1.0;
        let ty = 1.0;
        let matrix = [scalex, 0.0, 0.0, 0.0, -scaley, 0.0, tx, ty, 1.0];

        if as_square {
            self.square_program.draw(&self.ctx, verts, *offset, &matrix, point_size, color);
        } else {
            self.circle_program.draw(&self.ctx, verts, *offset, &matrix, point_size, color);
        };
    }
    pub fn draw_circles(
        &mut self,
        verts:&Buffer,
        game_dim: [f32; 2],
        color: &[f32; 4],
        offset: &[f32; 2],
        point_size: f32,
    ) {
        self.draw(Args {
            verts,
            game_dim,
            as_square: false,
            color,
            offset,
            point_size,
        })
    }
    pub fn draw_squares(
        &mut self,
        verts: &Buffer,
        game_dim: [f32; 2],
        color: &[f32; 4],
        offset: &[f32; 2],
        point_size: f32,
    ) {
        self.draw(Args {
            verts,
            game_dim,
            as_square: true,
            color,
            offset,
            point_size,
        })
    }

}



pub fn line(buffer: &mut Vec<Vertex>, radius: f32, start: [f32; 2], end: [f32; 2]) {
    let offsetx = end[0] - start[0];
    let offsety = end[1] - start[1];

    let dis_sqr = offsetx * offsetx + offsety * offsety;
    let dis = dis_sqr.sqrt();

    let normx = offsetx / dis;
    let normy = offsety / dis;

    let num = (dis / (radius)).floor() as usize;

    for i in 0..num {
        let x = start[0] + (i as f32) * normx * radius;
        let y = start[1] + (i as f32) * normy * radius;
        buffer.push(Vertex([x, y, 0.0]));
    }
}
