use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::{ensure, Result};
use chrono::Utc;

static SHIFT_JIS_2004: &[u8] = include_bytes!("../assets/shift_jis_2004.txt");

#[derive(Debug)]
enum Line {
    Unicode1(u32),
    Unicode2(Vec<u32>),
    DoubleByte(),
    Reserved(),
}

#[inline]
fn parse_hex(s: &str) -> Result<u32> {
    ensure!(s.starts_with("0x"), "too long");
    Ok(u32::from_str_radix(&s[2..], 16)?)
}

#[inline]
fn parse_unicode(s: &str) -> Result<Vec<u32>> {
    ensure!(s.starts_with("U+"), "invalid unicode");
    let v = s[2..]
        .split('+')
        .map(|x| u32::from_str_radix(x, 16))
        .collect::<std::result::Result<Vec<_>, _>>();
    Ok(v?)
}

fn parse_line(line: &str) -> Result<(u32, Line)> {
    let parts: Vec<&str> = line.split('#').collect();
    ensure!(parts.len() == 2, "invalid line");
    let before: Vec<&str> = parts[0].split(' ').filter(|s| !s.is_empty()).collect();
    let after: Vec<&str> = parts[1].split(' ').filter(|s| !s.is_empty()).collect();
    ensure!(before.len() == 1 || before.len() == 2, "invalid line");
    ensure!(after.len() >= 1, "invalid line");

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

fn sjis() -> Result<()> {
    let data = String::from_utf8(SHIFT_JIS_2004.to_vec())?;
    let lookup = data
        .replace("\t", " ")
        .lines()
        .filter(|x| !x.starts_with("#"))
        .map(parse_line)
        .flat_map(|x| x)
        .collect::<HashMap<_, _>>();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("shift_jis_table.rs");
    let mut output_file = File::create(&dest_path)?;

    writeln!(output_file, "// generated: {}", Utc::now())?;
    writeln!(
        output_file,
        "    static TO_UTF8: [Option<&'static str>; {}] = {{",
        256 * 256
    )?; 
    writeln!(
        output_file,
        "let mut table: [Option<&'static str>; {}] = [None; {}];",
        256 * 256,
        256 * 256
    )?;
    for entry in lookup.iter() {
        match entry.1 {
            Line::Unicode1(unicode) => {
                write!(output_file, "table[0x{:x}]=", entry.0)?;
                writeln!(output_file, "Some(\"\\u{{{:x}}}\");", *unicode)?;
            },
            Line::Unicode2(unicode) => {
                writeln!(
                    output_file,
                    "table[0x{:x}]=Some(\"{}\");",
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
    writeln!(output_file, "table")?;
    writeln!(output_file, "    }};")?;

    println!("cargo:rerun-if-changed=build/build.rs");

    Ok(())
}

fn main() { sjis().unwrap(); }
