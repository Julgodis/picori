#![no_main]
use libfuzzer_sys::fuzz_target;
use picori::string::ascii::AsciiEncodingTrait;

fuzz_target!(|data: &[u8]| {
    let _ = String::from_ascii(data);
});
