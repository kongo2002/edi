use gl33::{
    global_loader::*, ShaderType, GL_COMPILE_STATUS, GL_FRAGMENT_SHADER, GL_LINK_STATUS,
    GL_VERTEX_SHADER,
};

use crate::camera::Camera;
use crate::errors::EdiError;
use crate::render::V2;

const ERROR_BUFFER_SIZE: usize = 1024;

pub struct GL {
    programs: Vec<Shader>,
}

#[derive(Clone)]
pub struct Shader {
    program: u32,

    resolution_uniform: i32,
    camera_pos_uniform: i32,
    camera_scale_uniform: i32,
}

impl Shader {
    pub fn activate(&self, resolution: &V2, camera: &Camera) {
        glUseProgram(self.program);

        unsafe {
            glUniform2f(self.resolution_uniform, resolution.x, resolution.y);

            glUniform1f(self.camera_scale_uniform, camera.scale);
            glUniform2f(self.camera_pos_uniform, camera.pos.x, camera.pos.y);
        }
    }
}

impl GL {
    pub fn new() -> GL {
        GL {
            programs: Vec::new(),
        }
    }

    pub fn create_program(&mut self, vertex: &str, fragment: &str) -> Result<Shader, EdiError> {
        let program;

        let vertex_shader = Self::create_shader(GL_VERTEX_SHADER, vertex)?;
        let fragment_shader = Self::create_shader(GL_FRAGMENT_SHADER, fragment)?;

        unsafe {
            program = glCreateProgram();
            if program == 0 {
                return Err(EdiError::ProgramCreationFailed);
            }

            for shader_id in &[vertex_shader, fragment_shader] {
                glAttachShader(program, *shader_id);
            }

            glLinkProgram(program);

            let mut success = 0;
            glGetProgramiv(program, GL_LINK_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(ERROR_BUFFER_SIZE);
                let mut log_len = 0_i32;

                glGetProgramInfoLog(
                    program,
                    ERROR_BUFFER_SIZE as i32,
                    &mut log_len,
                    v.as_mut_ptr().cast(),
                );
                v.set_len(log_len.try_into().unwrap());

                return Err(EdiError::ProgramLinkingFailed(
                    String::from_utf8_lossy(&v).to_string(),
                ));
            }

            for shader_id in &[vertex_shader, fragment_shader] {
                glDeleteShader(*shader_id);
            }
        }

        let resolution_uniform = Self::get_location(program, "resolution")?;
        let camera_pos_uniform = Self::get_location(program, "camera_pos")?;
        let camera_scale_uniform = Self::get_location(program, "camera_scale")?;

        let shader = Shader {
            program,
            resolution_uniform,
            camera_pos_uniform,
            camera_scale_uniform,
        };

        self.programs.push(shader.clone());

        Ok(shader)
    }

    fn get_location(program: u32, name: &str) -> Result<i32, EdiError> {
        unsafe {
            let null_terminated = [name, "\0"].concat();
            let loc = glGetUniformLocation(program, null_terminated.as_bytes().as_ptr());
            if loc < 0 {
                Err(EdiError::UniformLookupFailed(name.to_string()))
            } else {
                Ok(loc)
            }
        }
    }

    fn create_shader(shader_type: ShaderType, shader_code: &str) -> Result<u32, EdiError> {
        let shader_id;
        unsafe {
            shader_id = glCreateShader(shader_type);
            if shader_id == 0 {
                return Err(EdiError::ShaderCreationFailed);
            }

            glShaderSource(
                shader_id,
                1,
                &(shader_code.as_bytes().as_ptr().cast()),
                &(shader_code.len().try_into().unwrap()),
            );

            glCompileShader(shader_id);

            let mut success = 0;
            glGetShaderiv(shader_id, GL_COMPILE_STATUS, &mut success);

            if success == 0 {
                const BUFFER_SIZE: usize = 1024;

                let mut v: Vec<u8> = Vec::with_capacity(BUFFER_SIZE);
                let mut log_len = 0_i32;
                glGetShaderInfoLog(
                    shader_id,
                    BUFFER_SIZE as i32,
                    &mut log_len,
                    v.as_mut_ptr().cast(),
                );
                v.set_len(log_len.try_into().unwrap());

                return Err(EdiError::ShaderCompileError(
                    String::from_utf8_lossy(&v).to_string(),
                ));
            }
        }

        Ok(shader_id)
    }
}
