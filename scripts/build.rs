mod gen_table;
mod shift_jis;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unable to parse value")]
    UnableToParseValue(#[from] std::num::ParseIntError),

    #[error("unable to write file")]
    UnableToWriteFile(#[from] std::io::Error),

    #[error("utf8 error")]
    FromUtf8(#[from] std::string::FromUtf8Error),

    #[error("unknown error")]
    Unknown,
}

fn main() {
    shift_jis::generate().expect("shift-jis table generation failed");

    println!("cargo:rerun-if-changed=scripts/shift_jis.rs");
    println!("cargo:rerun-if-changed=scripts/build.rs");
}
