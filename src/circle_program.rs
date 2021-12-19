use super::points::*;
use web_sys::WebGlBuffer;
use web_sys::{WebGlProgram, WebGl2RenderingContext};
use web_sys::WebGlUniformLocation;

impl CircleProgram{

    pub fn draw(&self,context:&WebGl2RenderingContext,buffer:&WebGlBuffer,
           offset:[f32;2],mmatrix:&[f32;9],point_size:f32,color:&[f32;4],vertices:&[Vertex]){
        context.use_program(Some(&self.program));
        
        context.uniform2f(Some(&self.offset),offset[0],offset[1]);
        context.uniform1f(Some(&self.point_size),point_size);
        context.uniform4fv_with_f32_array(Some(&self.bg),color);
        context.uniform_matrix3fv_with_f32_array(Some(&self.mmatrix),false,mmatrix);

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        context.vertex_attrib_pointer_with_i32(self.position as u32, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(0);

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let k:&[f32]=core::slice::from_raw_parts(vertices.as_ptr() as *const _,vertices.len()*3);

            let vert_array = js_sys::Float32Array::view(  k );

            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }


        context.draw_arrays(
            WebGl2RenderingContext::POINTS,
            0,
            vertices.len() as i32,
        );
        
    }

    pub fn new(context:&WebGl2RenderingContext,vs:&str,fs:&str)->Result<Self,String>{
        let vert_shader = compile_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            vs,
        )?;
        let frag_shader = compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            fs,
        )?;
        let program = link_program(&context, &vert_shader, &frag_shader)?;
    
        let mmatrix=context.get_uniform_location(&program,"mmatrix").ok_or("uniform err".to_string())?;
        let point_size=context.get_uniform_location(&program,"point_size").ok_or("uniform err".to_string())?;
        let offset=context.get_uniform_location(&program,"offset").ok_or("uniform err".to_string())?;
        let bg=context.get_uniform_location(&program,"bg").ok_or("uniform err".to_string())?;
        let position=  context.get_attrib_location(&program, "position");
        if position<0{
            return Err("attribute err".to_string());
        }
        let position=position as u32;

        Ok(CircleProgram{
            program,
            offset,
            mmatrix,
            point_size,
            bg,
            position
        })

    }
}


pub struct CircleProgram{
    program: WebGlProgram,
    offset:WebGlUniformLocation,
    mmatrix:WebGlUniformLocation,
    point_size:WebGlUniformLocation,
    bg:WebGlUniformLocation,
    position:u32
}