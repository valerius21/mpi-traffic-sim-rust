//! Main Crate Error

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// For starter, to remove as code matures.
    #[error("Generic error: {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Vector is empty")]
    EmptyVector(String),

    #[error("bincode error: {0}")]
    Bincode(#[from] bincode::Error),
}
