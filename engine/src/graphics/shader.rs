extern crate gl;

#[derive(PartialEq, Clone, Debug)]
pub struct Shader {
    program: u32,
    pub uniforms: Vec<Uniform>,
}

impl Shader {
    pub fn new(vertex_source: &str, fragment_source: &str) -> Self {
        let vertex_shader;
        let fragment_shader;

        // Vertex
        unsafe {
            vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let ptr = vertex_source.as_bytes().as_ptr();
            let ptr_i8: *const i8 = std::mem::transmute(ptr);
            gl::ShaderSource(
                vertex_shader,
                1,
                &ptr_i8,
                &(vertex_source.len() as gl::types::GLint),
            );
            gl::CompileShader(vertex_shader);
            let mut success = 0;
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success as *mut i32);
            assert!(success > 0);
            if success == 0 {
                panic!("vertex shader error!")
            }
        }

        // Fragment
        unsafe {
            fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let ptr = fragment_source.as_bytes().as_ptr();
            let ptr_i8: *const i8 = std::mem::transmute(ptr);
            gl::ShaderSource(
                fragment_shader,
                1,
                &ptr_i8,
                &(fragment_source.len() as gl::types::GLint),
            );
            gl::CompileShader(fragment_shader);
            let mut success = 0;
            gl::GetShaderiv(
                fragment_shader,
                gl::COMPILE_STATUS,
                &mut success as *mut i32,
            );
            // let mut l = 0;
            // let mut infoLog: [i8; 300] = [0; 300];
            // gl::GetShaderInfoLog(fragment_shader, 300, &mut l, &mut infoLog[0]);
            // println!(
            //     "{:?}",
            //     String::from_raw_parts(infoLog.as_mut_ptr() as *mut u8, l as usize, 300)
            // );
            assert!(success > 0);
        }

        // Program
        let shader_program;
        unsafe {
            shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);
            let mut success = 0;
            let sp: *mut i32 = &mut success;
            //TODO: GetProgramInfoLog?
            gl::GetProgramiv(shader_program, gl::LINK_STATUS, sp);
            assert!(success > 0);

            gl::UseProgram(shader_program);
            gl::DetachShader(shader_program, vertex_shader);
            gl::DeleteShader(vertex_shader);
            gl::DetachShader(shader_program, fragment_shader);
            gl::DeleteShader(fragment_shader);
        }

        // get uniforms
        let mut uniforms: Vec<Uniform> = Vec::new();
        unsafe {
            let mut active_uniforms: gl::types::GLint = 0;
            gl::GetProgramiv(
                shader_program,
                gl::ACTIVE_UNIFORMS,
                &mut active_uniforms as *mut gl::types::GLint,
            );

            println!("program uniform count : {}", active_uniforms);
            const MAX_NAME_LENGTH: i32 = 128;
            let mut name: [gl::types::GLchar; MAX_NAME_LENGTH as usize] =
                [0; MAX_NAME_LENGTH as usize];
            let mut length: gl::types::GLsizei = 0;
            let mut size: gl::types::GLsizei = 0;
            let mut type_: gl::types::GLenum = 0;
            for i in 0..active_uniforms {
                gl::GetActiveUniform(
                    shader_program,
                    i as u32,
                    MAX_NAME_LENGTH,
                    &mut length as *mut gl::types::GLsizei,
                    &mut size as *mut gl::types::GLsizei,
                    &mut type_ as *mut gl::types::GLenum,
                    &mut name[0] as *mut gl::types::GLchar,
                );
                println!("program uniforms:");
                println!("{:?}", name);

                // todo: this is pretty bad
                let uniform_name = &name[0..(length as usize)];
                let u8slice = &*(uniform_name as *const [i8] as *const [u8]);
                let location =
                    gl::GetUniformLocation(shader_program, uniform_name.as_ptr() as *const i8);

                let info = Uniform {
                    name: String::from_utf8_lossy(u8slice).to_string(),
                    uniform_type: UniformType::FLOAT,
                    shader_type: ShaderType::FRAGMENT,
                    location,
                };
                uniforms.push(info);
            }
        }

        Shader {
            program: shader_program,
            uniforms,
        }
    }

    pub fn set(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }
}
#[derive(PartialEq, Clone, Debug)]
enum ShaderType {
    NONE,
    VERTEX,
    FRAGMENT,
}
#[derive(PartialEq, Clone, Debug)]
pub enum UniformType {
    NONE,
    FLOAT,
    FLOAT2,
    FLOAT3,
    FLOAT4,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Uniform {
    pub name: String,
    pub location: i32,
    pub uniform_type: UniformType,
    shader_type: ShaderType,
}
