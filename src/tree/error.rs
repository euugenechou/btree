use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Serde(#[from] bincode::Error),

    #[error("allocator error")]
    Allocator,

    #[error("storage error")]
    Storage,
}
