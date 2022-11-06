use thiserror::Error;

#[derive(Error, Debug)]
pub enum DolError {
    #[error("Invalid header size: requires `0x100` bytes, got `{size:?}` bytes")]
    InvalidHeaderSize { size: usize },

    #[error("Invalid text section `{section:?}`: offset out of bounds")]
    TextSectionOutOfBounds { section: usize },

    #[error("Invalid data section `{section:?}`: offset out of bounds")]
    DataSectionOutOfBounds { section: usize },
}

#[derive(Error, Debug)]
pub enum DecompressionError {
    #[error("Invalid header")]
    InvalidHeader(),

    #[error("Missing data")]
    MissingData(),

    #[error("Invalid source offset")]
    InvalidSourceOffset(),

    #[error("Invalid destination offset")]
    InvalidDestinationOffset(),
}

#[derive(Error, Debug)]
pub enum PicoriError {
    #[error("Integer overflow occured")]
    IntegerOverflow(),

    #[error("Parse error: {error:?}")]
    Dol {
        #[from]
        error: DolError,
    },

    #[error("Decompression error: {error:?}")]
    Decompression {
        #[from]
        error: DecompressionError,
    },
}
