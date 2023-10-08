use gl33::{global_loader::*, ShaderType, GL_COMPILE_STATUS, GL_LINK_STATUS};

use crate::errors::EdiError;

const ERROR_BUFFER_SIZE: usize = 1024;

pub struct GL {
    pub id: u32,
}

impl GL {
    pub fn create_program(shaders: &[u32]) -> Result<GL, EdiError> {
        let id;

        unsafe {
            id = glCreateProgram();
            if id == 0 {
                return Err(EdiError::ProgramCreationFailed);
            }

            for shader_id in shaders {
                glAttachShader(id, *shader_id);
            }

            glLinkProgram(id);

            let mut success = 0;
            glGetProgramiv(id, GL_LINK_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(ERROR_BUFFER_SIZE);
                let mut log_len = 0_i32;

                glGetProgramInfoLog(
                    id,
                    ERROR_BUFFER_SIZE as i32,
                    &mut log_len,
                    v.as_mut_ptr().cast(),
                );
                v.set_len(log_len.try_into().unwrap());

                return Err(EdiError::ProgramLinkingFailed(
                    String::from_utf8_lossy(&v).to_string(),
                ));
            }

            for shader_id in shaders {
                glDeleteShader(*shader_id);
            }
        }

        Ok(GL { id })
    }

    pub fn use_program(&self) {
        glUseProgram(self.id)
    }

    pub fn create_shader(shader_type: ShaderType, shader_code: &str) -> Result<u32, EdiError> {
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
