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

fn parse_hex(s: &str) -> Result<u32> {
    ensure!(s.starts_with("0x"), "too long");
    Ok(u32::from_str_radix(&s[2..], 16)?)
}

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

    for (key, value) in lookup.iter() {
        // println!("{}: {:?}", key, value);
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("shift_jis_table.rs");
    let mut output_file = File::create(&dest_path)?;

    writeln!(output_file, "// generated: {}", Utc::now())?;

    writeln!(output_file, "use lazy_static::lazy_static;")?;
    writeln!(output_file, "lazy_static! {{")?;
    writeln!(
        output_file,
        "    static ref TO_UTF8: Vec<Option<&'static str>> = {{"
    )?;
    writeln!(
        output_file,
        "        let mut table: Vec<Option<&'static str>> = vec![None; 256 * 256];"
    )?;

    for entry in lookup.iter() {
        match entry.1 {
            Line::Unicode1(unicode) => {
                write!(output_file, "            table[0x{:04x}] = ", entry.0)?;
                writeln!(output_file, "Some(\"\\u{{{:x}}}\");", *unicode)?;
            },
            Line::Unicode2(unicode) => {
                writeln!(
                    output_file,
                    "        table[0x{:04x}] = Some(\"{}\");",
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

    writeln!(output_file, "        table")?;
    writeln!(output_file, "    }};")?;
    writeln!(output_file, "}}")?;

    // writeln!(output_file, "enum Kind {{")?;
    // writeln!(output_file, "  S(),")?;
    // writeln!(output_file, "  M(char),")?;
    // writeln!(output_file, "  M2(char, char),")?;
    // writeln!(output_file, "  D(&'static [&str]),")?;
    // writeln!(output_file, "  I(),")?;
    // writeln!(output_file, "}}")?;
    //
    // writeln!(output_file, "use Kind::*;")?;
    //
    // let mut double_lookup: HashMap<u8, HashMap<u8, Vec<u32>>> = HashMap::new();
    // for entry in lookup.iter() {
    // match entry.1 {
    // Line::DoubleByte() => {
    // let key = (entry.0 & 0xff) as u8;
    // double_lookup.insert(key, HashMap::new());
    // },
    // _ => {},
    // }
    // }
    //
    // for entry in lookup.iter() {
    // let second = (entry.0 & 0xff) as u8;
    // let first = (entry.0 >> 8) as u8;
    // match entry.1 {
    // Line::Unicode1(unicode) => {
    // double_lookup
    // .get_mut(&first)
    // .and_then(|x| x.insert(second, vec![*unicode]));
    // },
    // Line::Unicode2(unicode) => {
    // double_lookup
    // .get_mut(&first)
    // .and_then(|x| x.insert(second, unicode.clone()));
    // },
    // _ => {},
    // }
    // }
    //
    // for entry in double_lookup.iter() {
    // writeln!(output_file, "static SECOND_{:X}: [&str; 256] = [", entry.0)?;
    // for i in 0..256 {
    // let key = i as u8;
    // if let Some(value) = entry.1.get(&key) {
    // match value.len() {
    // 1 => write!(output_file, "\"\\u{{{:x}}}\",", value[0])?,
    // 2 => write!(
    // output_file,
    // "\"\\u{{{:x}}}\\u{{{:x}}}\",",
    // value[0], value[1]
    // )?,
    // _ => unimplemented!("invalid line"),
    // }
    // } else {
    // write!(output_file, "\"\",")?;
    // }
    // if(i + 1) % 64 == 0 {
    // writeln!(output_file)?;
    // }
    // }
    // writeln!(output_file, "];")?;
    // }
    //
    // write!(output_file, "static FIRST_BYTE: [Kind; 256] = [")?;
    // for i in 0..256 {
    // let key = i as u32;
    // if let Some(_value) = lookup.get(&key) {
    // match _value {
    // Line::Unicode1(unicode) => {
    // if *unicode == key {
    // write!(output_file, "S(),")?;
    // } else {
    // write!(output_file, "M('\\u{{{:x}}}'),", *unicode)?;
    // }
    // },
    // Line::DoubleByte() => {
    // write!(output_file, "D(&SECOND_{:X}),", key)?;
    // },
    // Line::Reserved() => {
    // write!(output_file, "I(),")?;
    // },
    // _ => unimplemented!("invalid line"),
    // }
    // } else {
    // write!(output_file, "I(),")?;
    // }
    // }
    // writeln!(output_file, "];")?;
    Ok(())
}

fn main() { sjis().unwrap(); }
