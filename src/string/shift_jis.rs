// http://x0213.org/codetable/index.en.html

use std::result::Result;

use super::StringError;

include!(concat!(env!("OUT_DIR"), "/shift_jis_table.rs"));

pub trait ShiftJis2004 {
    fn from_shift_jis_2004(input: &[u8]) -> Result<String, StringError>;
}

impl ShiftJis2004 for String {
    fn from_shift_jis_2004(input: &[u8]) -> Result<String, StringError> {
        let mut i = 0;
        let mut output = String::new();
        while i < input.len() {
            let first = input[i] as usize;
            i += 1;

            if first == 0 {
                break;
            }

            if let Some(value) = TO_UTF8[first] {
                output.push_str(value);
                continue;
            }

            let second = input[i] as usize;
            i += 1;

            let code = first * 256 + second;
            if let Some(value) = TO_UTF8[code] {
                output.push_str(value);
                continue;
            }

            unreachable!();
        }
        Ok(output)
    }
}
