use std::panic::Location;

/// Enum for possible decompression problems that can occur.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum DecompressionProblem {
    /// Reading decompression header failed.
    #[error("invalid header: {0} at {1}")]
    InvalidHeader(&'static str, &'static Location<'static>),

    /// Decompressing data failed.
    #[error("invalid data: {0} at {1}")]
    InvalidData(&'static str, &'static Location<'static>),

    /// Decompression failed because of an unexpected end of data.
    #[error("unexpected EOD: {0}")]
    UnexpectedEndOfData(&'static Location<'static>),

    /// Decompression failed because of an unexpected destination size.
    #[error("invalid decompression size: {0}")]
    InvalidDecompressedSize(&'static Location<'static>),
}
