#[derive(thiserror::Error, Debug)]
pub enum EdiError {
    #[error("SDL error: {0}")]
    SdlError(String),
    #[error("IO error: {0}")]
    IoError(#[from]std::io::Error),
    #[error("freeype error: {0}")]
    FreeTypeError(#[from]freetype::Error)
}
