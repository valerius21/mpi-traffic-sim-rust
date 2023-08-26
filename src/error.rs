//! Main Crate Error

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// For starter, to remove as code matures.
    #[error("Generic error: {0}")]
    Generic(String),
    /// For starter, to remove as code matures.
    // #[error("Static error: {0}")]
    // Static(&'static str),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Vector is empty")]
    EmptyVector(String),

    #[error("No Weight found for: {0}")]
    NoWeightFound(String),

    #[error("No inner Edge")]
    NoInnerEdge(String),

    #[error("Element not in Vector: {0}")]
    ElementNotInVector(String),

    #[error("bincode error: {0}")]
    Bincode(#[from] bincode::Error),
}
