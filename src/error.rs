use thiserror::Error;

use crate::BufType;

/// The error type for interactions with this library.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The number of allocated buffers was less than the amount requested.
    #[error("failed to allocate {required_count} buffers (driver only allocated {actual_count})")]
    BufferAllocationFailed {
        required_count: u32,
        actual_count: u32,
    },
    /// The requested buffer type is not supported.
    #[error("unsupported buffer type {0:?}")]
    UnsupportedBufferType(BufType),
    /// An underlying I/O error has occurred.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<nix::Error> for Error {
    fn from(error: nix::Error) -> Self {
        Error::Io(std::io::Error::from_raw_os_error(error as i32))
    }
}
