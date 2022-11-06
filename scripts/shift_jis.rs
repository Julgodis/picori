use std::collections::HashMap;
use std::env;
use std::fs::{OpenOptions};
use std::io::{BufWriter, Write};
use std::num::ParseIntError;
use std::path::Path;
use std::result::Result;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unable to parse value")]
    UnableToParseValue(#[from] ParseIntError),

    #[error("unable to write file")]
    UnableToWriteFile(#[from] std::io::Error),

    #[error("utf8 error")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

#[derive(Debug)]
enum Line {
    Unicode1(u32),
    Unicode2(Vec<u32>),
    DoubleByte(),
    Reserved(),
}

#[inline]
fn parse_hex(s: &str) -> Result<u32, Error> {
    assert!(s.starts_with("0x"), "too long");
    Ok(u32::from_str_radix(&s[2..], 16)?)
}

#[inline]
fn parse_unicode(s: &str) -> Result<Vec<u32>, Error> {
    assert!(s.starts_with("U+"), "invalid unicode");
    let v = s[2..]
        .split('+')
        .map(|x| u32::from_str_radix(x, 16))
        .collect::<Result<Vec<_>, _>>();
    Ok(v?)
}

fn parse_line(line: &str) -> Result<(u32, Line), Error> {
    let parts: Vec<&str> = line.split('#').collect();
    assert!(parts.len() == 2, "invalid line");
    let before: Vec<&str> = parts[0].split(' ').filter(|s| !s.is_empty()).collect();
    let after: Vec<&str> = parts[1].split(' ').filter(|s| !s.is_empty()).collect();
    assert!(before.len() == 1 || before.len() == 2, "invalid line");
    assert!(after.len() >= 1, "invalid line");

    let code = parse_hex(before[0])?;
    if before.len() == 1 {
        match after[0] {
            "<doublebytes>" => Ok((code, Line::DoubleByte())),
            "<reserved>" => Ok((code, Line::Reserved())),
            _ => unimplemented!("invalid line"),
        }
    } else {
        let unicode = parse_unicode(before[1])?;
        if unicode.len() == 1 {
            Ok((code, Line::Unicode1(unicode[0])))
        } else {
            Ok((code, Line::Unicode2(unicode)))
        }
    }
}

fn write_file(path: &Path, hashmap: &HashMap<u32, Line>) -> Result<(), Error> {
    let mut output_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    let mut buffer = BufWriter::new(&mut output_file);

    writeln!(
        buffer,
        "static TO_UTF8:[Option<&'static str>;{}]={{",
        256 * 256
    )?;
    writeln!(
        buffer,
        "let mut t:[Option<&'static str>;{}]=[None;{}];",
        256 * 256,
        256 * 256
    )?;
    for entry in hashmap.iter() {
        match entry.1 {
            Line::Unicode1(unicode) => {
                writeln!(
                    buffer,
                    "t[0x{:x}]=Some(\"\\u{{{:x}}}\");",
                    entry.0, *unicode
                )?;
            },
            Line::Unicode2(unicode) => {
                writeln!(
                    buffer,
                    "t[0x{:x}]=Some(\"{}\");",
                    entry.0,
                    unicode
                        .iter()
                        .map(|x| format!("\\u{{{:x}}}", *x))
                        .collect::<String>()
                )?;
            },
            _ => {},
        }
    }
    writeln!(buffer, "t")?;
    writeln!(buffer, "}};")?;

    Ok(())
}

static SHIFT_JIS_2004: &[u8] = include_bytes!("../assets/shift_jis_2004.txt");

pub fn generate() -> Result<(), Error> {
    let data = String::from_utf8(SHIFT_JIS_2004.to_vec())?;
    let lookup = data
        .replace("\t", " ")
        .lines()
        .filter(|x| !x.starts_with("#"))
        .map(parse_line)
        .flat_map(|x| x)
        .collect::<HashMap<_, _>>();

    let dir = env::var_os("OUT_DIR").unwrap();
    let path = Path::new(&dir).join("shift_jis_table.rs");
    write_file(&path, &lookup)?;

    Ok(())
}
