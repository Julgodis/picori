#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate picori;

fuzz_target!(|data: &[u8]| {
    let _ = picori::dol::Dol::from_bytes(data);
});
