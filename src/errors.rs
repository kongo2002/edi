#[derive(thiserror::Error, Debug)]
pub enum EdiError {
    #[error("SDL error: {0}")]
    SdlError(String),
}
