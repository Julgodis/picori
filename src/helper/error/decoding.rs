use std::panic::Location;

/// Enum for possible decoding problems that can occur.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum DecodingProblem {
    /// Decoding try to create a character that is not valid in UTF-8.
    #[error("invalid code point: {0} at {1}")]
    InvalidCodePoint(usize, &'static Location<'static>),

    /// Invalid byte sequence.
    #[error("invalid byte: {0} at {1}")]
    InvalidByte(u8, &'static Location<'static>),

    /// Decoding failed due to unexpected end of data, .i.e, it requires more
    /// data to fully decode.
    #[error("unexpected EOD: {0}")]
    UnexpectedEndOfData(&'static Location<'static>),
}
