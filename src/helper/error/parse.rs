use std::panic::Location;

/// Enum for possible parse problems that can occur.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ParseProblem {
    /// Invalid magic found while parsing.
    #[error("invalid magic: {0}")]
    InvalidMagic(&'static str, &'static Location<'static>),

    /// Invalid value range found while parsing.
    #[error("invalid range: {0}")]
    InvalidRange(&'static str, &'static Location<'static>),

    /// Invalid header found while parsing.
    #[error("invalid header: {0}")]
    InvalidHeader(&'static str, &'static Location<'static>),

    /// Unable to parse data.
    #[error("invalid data: {0}")]
    InvalidData(&'static str, &'static Location<'static>),

    /// Unsupported version.
    #[error("unsupported version: {0} at {1}")]
    UnsupportedVersion(usize, &'static Location<'static>),
}
