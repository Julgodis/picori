#![no_main]
use libfuzzer_sys::fuzz_target;
use picori::AsciiIteratorExt;
extern crate picori;

fuzz_target!(|data: &[u8]| {
    let _ = data.iter().ascii().collect::<Result<String, _>>();
});
