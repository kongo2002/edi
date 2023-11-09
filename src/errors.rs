#[derive(thiserror::Error, Debug)]
pub enum EdiError {
    #[error("SDL error: {0}")]
    SdlError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("freeype error: {0}")]
    FreeTypeError(#[from] freetype::Error),
    #[error("shader creation failed")]
    ShaderCreationFailed,
    #[error("shader compilation failed: {0}")]
    ShaderCompileError(String),
    #[error("OpenGL program creation failed")]
    ProgramCreationFailed,
    #[error("program linking failed: {0}")]
    ProgramLinkingFailed(String),
    #[error("lookup of uniform '{0}' failed")]
    UniformLookupFailed(String),
    #[error("terminated without success")]
    Cancelled,
}
