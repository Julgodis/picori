pub mod alignment;
mod endian;
mod error;

mod parser;
mod reader;
mod seeker;
mod writer;

pub use error::build::BuildProblem;
pub use error::compression::CompressionProblem;
pub use error::decoding::DecodingProblem;
pub use error::decompression::DecompressionProblem;
pub use error::encoding::EncodingProblem;
pub use error::parse::ParseProblem;
pub(crate) use error::{ensure, ProblemLocation};
pub use error::{Error, Result};
pub(crate) use parser::*;

pub use seeker::Seeker;
pub use reader::Reader;
pub use writer::Writer;
pub use parser::Parser;