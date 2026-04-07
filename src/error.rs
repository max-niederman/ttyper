use thiserror::Error;

#[derive(Error, Debug)]
pub enum TtyperError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Application error: {0}")]
    Application(String),

    #[error("Content error: {0}")]
    Content(String),
}

pub type Result<T> = std::result::Result<T, TtyperError>;
