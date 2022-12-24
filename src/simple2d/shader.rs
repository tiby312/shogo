use web_sys::WebGlShader;
use web_sys::WebGlUniformLocation;
use web_sys::{WebGl2RenderingContext, WebGlProgram};

///
/// A webgl2 buffer that automatically deletes itself when dropped.
///
pub struct Buffer {
    pub(crate) buffer: web_sys::WebGlBuffer,
    pub(crate) num_verts: usize,
    pub(crate) ctx: WebGl2RenderingContext,
}
impl Buffer {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<Self, String> {
        let buffer = ctx.create_buffer().ok_or("failed to create buffer")?;
        Ok(Buffer {
            buffer,
            num_verts: 0,
            ctx: ctx.clone(),
        })
    }
}
impl Drop for Buffer {
    fn drop(&mut self) {
        self.ctx.delete_buffer(Some(&self.buffer));
    }
}

impl GlProgram {
    pub fn draw(
        &self,
        buffer: &Buffer,
        primitive: u32,
        offset: [f32; 2],
        mmatrix: &[f32; 9],
        point_size: f32,
        color: &[f32; 4],
    ) {
        if buffer.num_verts == 0 {
            return;
        }

        let context = &buffer.ctx;

        context.use_program(Some(&self.program));

        context.uniform2f(Some(&self.offset), offset[0], offset[1]);
        context.uniform1f(Some(&self.point_size), point_size);
        context.uniform4fv_with_f32_array(Some(&self.bg), color);

        context.uniform_matrix3fv_with_f32_array(Some(&self.mmatrix), false, mmatrix);

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer.buffer));

        context.vertex_attrib_pointer_with_i32(
            self.position as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        context.enable_vertex_attrib_array(0);

        context.draw_arrays(primitive, 0, buffer.num_verts as i32);
    }

    pub fn new(context: &WebGl2RenderingContext, vs: &str, fs: &str) -> Result<Self, String> {
        let vert_shader = compile_shader(context, WebGl2RenderingContext::VERTEX_SHADER, vs)?;
        let frag_shader = compile_shader(context, WebGl2RenderingContext::FRAGMENT_SHADER, fs)?;
        let program = link_program(context, &vert_shader, &frag_shader)?;

        context.delete_shader(Some(&vert_shader));
        context.delete_shader(Some(&frag_shader));

        let mmatrix = context
            .get_uniform_location(&program, "mmatrix")
            .ok_or_else(|| "uniform err".to_string())?;
        let point_size = context
            .get_uniform_location(&program, "point_size")
            .ok_or_else(|| "uniform err".to_string())?;
        let offset = context
            .get_uniform_location(&program, "offset")
            .ok_or_else(|| "uniform err".to_string())?;
        let bg = context
            .get_uniform_location(&program, "bg")
            .ok_or_else(|| "uniform err".to_string())?;
        let position = context.get_attrib_location(&program, "position");
        if position < 0 {
            return Err("attribute err".to_string());
        }
        let position = position as u32;

        Ok(GlProgram {
            program,
            offset,
            mmatrix,
            point_size,
            bg,
            position,
        })
    }
}

pub struct GlProgram {
    pub(crate) program: WebGlProgram,
    offset: WebGlUniformLocation,
    mmatrix: WebGlUniformLocation,
    point_size: WebGlUniformLocation,
    bg: WebGlUniformLocation,
    position: u32,
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
