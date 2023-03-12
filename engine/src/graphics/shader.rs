extern crate gl;

pub struct Shader {
    program: u32,
}

impl Shader {
    pub fn new(vertexSource: &str, fragmentSource: &str) -> Self {
        let mut vertexShader = 0;
        let mut fragmentShader = 0;

        // Vertex
        unsafe {
            vertexShader = gl::CreateShader(gl::VERTEX_SHADER);
            let ptr = vertexSource.as_bytes().as_ptr();
            let ptr_i8: *const i8 = std::mem::transmute(ptr);
            gl::ShaderSource(
                vertexShader,
                1,
                &ptr_i8,
                &(vertexSource.len() as gl::types::GLint),
            );
            gl::CompileShader(vertexShader);
            let mut success = 0;
            gl::GetShaderiv(vertexShader, gl::COMPILE_STATUS, &mut success as *mut i32);
            assert!(success > 0);
            if success == 0 {
                panic!("vertex shader error!")
            }
        }

        // Fragment
        unsafe {
            fragmentShader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let ptr = fragmentSource.as_bytes().as_ptr();
            let ptr_i8: *const i8 = std::mem::transmute(ptr);
            gl::ShaderSource(
                fragmentShader,
                1,
                &ptr_i8,
                &(fragmentSource.len() as gl::types::GLint),
            );
            gl::CompileShader(fragmentShader);
            let mut success = 0;
            gl::GetShaderiv(fragmentShader, gl::COMPILE_STATUS, &mut success as *mut i32);
            assert!(success > 0);
        }

        // Program
        let mut shaderProgram = 0;
        unsafe {
            shaderProgram = gl::CreateProgram();
            gl::AttachShader(shaderProgram, vertexShader);
            gl::AttachShader(shaderProgram, fragmentShader);
            gl::LinkProgram(shaderProgram);
            let mut success = 0;
            let mut sp: *mut i32 = &mut success;
            gl::GetProgramiv(shaderProgram, gl::LINK_STATUS, sp);
            assert!(success > 0);

            gl::UseProgram(shaderProgram);
            gl::DeleteShader(vertexShader);
            gl::DeleteShader(fragmentShader);
        }

        Shader {
            program: shaderProgram,
        }
    }
}
