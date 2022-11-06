use std::env;
use std::fs::OpenOptions;
use std::io::{BufWriter};
use std::path::Path;
use std::result::Result;

use crate::gen_table::GenTable;
use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
enum Version {
    SinceBeginningOfTime = 0,
    Since1983 = 1983,
    Since1997 = 1997,
    Since2000 = 2000,
    Since2004 = 2004,
}

#[derive(Debug)]
enum Value {
    Unicode1(u32),
    Unicode2(u32, u32),
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

struct Data {
    byte0:   u8,
    byte1:   u8,
    version: Version,
    value:   Value,
}

fn parse_line(line: &str) -> Result<Data, Error> {
    let parts: Vec<&str> = line.split('#').collect();
    assert!(parts.len() == 2, "invalid line");
    let before: Vec<&str> = parts[0].split(' ').filter(|s| !s.is_empty()).collect();
    let after: Vec<&str> = parts[1].split(' ').filter(|s| !s.is_empty()).collect();
    assert!(before.len() == 1 || before.len() == 2, "invalid line");
    assert!(!after.is_empty(), "invalid line");

    let mut version = Version::SinceBeginningOfTime;
    for v in &after {
        match *v {
            "[1983]" => version = Version::Since1983,
            "[1997]" => version = Version::Since1997,
            "[2000]" => version = Version::Since2000,
            "[2004]" => version = Version::Since2004,
            _ => {
                continue;
            },
        }

        break;
    }

    let code = parse_hex(before[0])?;
    let byte0 = (code >> 8) as u8;
    let byte1 = code as u8;
    if byte0 == 0x00 {
        return Err(Error::Unknown);
    }

    if before.len() == 1 {
        match after[0] {
            "<reserved>" => Ok(Data {
                byte0,
                byte1,
                version,
                value: Value::Reserved(),
            }),
            _ => unimplemented!("invalid data"),
        }
    } else {
        let unicode = parse_unicode(before[1])?;
        if unicode.len() == 1 {
            assert!(unicode[0] <= 0x10FFFF, "invalid unicode");
            Ok(Data {
                byte0,
                byte1,
                version,
                value: Value::Unicode1(unicode[0]),
            })
        } else if unicode.len() == 2 {
            assert!(unicode[0] <= 0x10FFFF, "invalid unicode");
            assert!(unicode[1] <= 0x10FFFF, "invalid unicode");
            Ok(Data {
                byte0,
                byte1,
                version,
                value: Value::Unicode2(unicode[0], unicode[1]),
            })
        } else {
            unimplemented!("invalid data");
        }
    }
}

fn generate_table(
    path: &Path,
    name: &'static str,
    version: Version,
    data: &[Data],
) -> Result<(), Error> {
    let data = data
        .iter()
        .filter(|d| d.version <= version)
        .filter(|d| match d.value {
            Value::Unicode1(_) => true,
            Value::Unicode2(_, _) => true,
            Value::Reserved() => false,
        })
        .collect::<Vec<_>>();
    let lookup_byte = (0x80..0xA0).chain(0xE0..0xFF).collect::<Vec<_>>();

    let mut single_lookup = Vec::<u32>::new();
    let mut double_lookup = Vec::<(u32, u32)>::new();
    let mut table_lookup = vec![(0_u8, 0_u8, 0_usize); 256];
    for byte0 in lookup_byte {
        let mut u1 = data
            .iter()
            .filter(|x| x.byte0 == byte0)
            .filter_map(|x| match x.value {
                Value::Unicode1(u) => Some((x.byte1, u)),
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut u2 = data
            .iter()
            .filter(|x| x.byte0 == byte0)
            .filter_map(|x| match x.value {
                Value::Unicode2(u1, u2) => Some((x.byte1, u1, u2)),
                _ => None,
            })
            .collect::<Vec<_>>();

        u1.sort_by(|a, b| a.0.cmp(&b.0));
        u2.sort_by(|a, b| a.0.cmp(&b.0));

        let first = u1.iter().map(|x| x.0).chain(u2.iter().map(|x| x.0)).min();
        let last = u1.iter().map(|x| x.0).chain(u2.iter().map(|x| x.0)).max();
        if first.is_none() || last.is_none() {
            continue;
        }

        let first = first.unwrap();
        let last = last.unwrap();
        let count = last - first + 1;
        let offset = single_lookup.len();
        single_lookup.extend(vec![0; count as usize]);
        table_lookup[byte0 as usize] = (first, last, offset);

        for (byte1, unicode) in u1 {
            let relative = (byte1 - first) as usize;
            let index = offset + relative;
            assert!(unicode != 0);
            assert!(single_lookup[index] == 0);
            single_lookup[index] = unicode;
        }

        for (byte1, unicode0, unicode1) in u2 {
            let relative = (byte1 - first) as usize;
            let index = offset + relative;
            let value = 0x8000_0000 | (double_lookup.len() as u32);
            assert!(single_lookup[index] == 0);
            single_lookup[index] = value;
            double_lookup.push((unicode0, unicode1));
        }
    }

    let mut output_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    let mut buffer = BufWriter::new(&mut output_file);

    table_lookup.gen_table(format!("{name}_UTF8_T"), &mut buffer)?;
    single_lookup.gen_table(format!("{name}_UTF8_S"), &mut buffer)?;
    if !double_lookup.is_empty() {
        double_lookup.gen_table(format!("{name}_UTF8_D"), &mut buffer)?;
    }

    Ok(())
}

static SHIFT_JIS_1983_2004: &str = include_str!("../assets/build/shift-jis.txt");

pub fn generate() -> Result<(), Error> {
    let data = SHIFT_JIS_1983_2004
        .replace('\t', " ")
        .lines()
        .filter(|x| !x.starts_with('#'))
        .flat_map(parse_line)
        .collect::<Vec<_>>();

    let dir = env::var_os("OUT_DIR").unwrap();
    let path_1997 = Path::new(&dir).join("shift_jis_1997.rs");
    generate_table(&path_1997, "SJIS_1997", Version::Since1997, &data)?;
    let path_2004 = Path::new(&dir).join("shift_jis_2004.rs");
    generate_table(&path_2004, "SJIS_2004", Version::Since2004, &data)?;

    Ok(())
}
