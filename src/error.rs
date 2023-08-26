//! Main Crate Error

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    EmptyVector(#[from] EmptyVectorError),
}

#[derive(Debug, Clone)]
pub struct EmptyVectorError;

impl std::fmt::Display for EmptyVectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "The vector is empty")
    }
}

impl std::error::Error for EmptyVectorError {}
