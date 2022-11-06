// http://x0213.org/codetable/index.en.html

use anyhow::{ensure, Result};

include!(concat!(env!("OUT_DIR"), "/shift_jis_table.rs"));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftJisStandard {
    X2004,
}

pub trait ShiftJisDecoding {
    fn from_shift_jis(input: &[u8], standard: ShiftJisStandard) -> Result<String>;
}

impl ShiftJisDecoding for String {
    fn from_shift_jis(input: &[u8], standard: ShiftJisStandard) -> Result<String> {
        ensure!(standard == ShiftJisStandard::X2004, "unsupported standard");

        let mut i = 0;
        let mut output = String::new();
        while i < input.len() {
            let first = input[i] as usize;
            i += 1;

            if let Some(value) = TO_UTF8[first] {
                println!("1 {}: {}", first, value);
                output.push_str(value);
                continue;
            }
            
            let second = input[i] as usize;
            i += 1;

            let code = first * 256 + second;
            if let Some(value) = TO_UTF8[code] {
                println!("2 {}: {}", first, value);
                output.push_str(value);
                continue;
            }

            unreachable!();
        }
        Ok(output)
    }
}
