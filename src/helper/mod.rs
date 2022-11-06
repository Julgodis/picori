pub mod alignment;
mod endian;
mod error;

mod parser;
mod reader;
mod seeker;

pub use error::build::BuildProblem;
pub(crate) use error::ensure;
pub use error::parse::ParseProblem;
pub use error::{
    CompressionProblem, DecodingProblem, DecompressionProblem, EncodingProblem, Error,
    ProblemLocation, Result,
};
pub(crate) use parser::*;
pub(crate) use reader::*;
pub(crate) use seeker::*;
