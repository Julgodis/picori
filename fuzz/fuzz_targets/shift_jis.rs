#![no_main]
use libfuzzer_sys::fuzz_target;
use picori::string::shift_jis::ShiftJis2004;
extern crate picori;

fuzz_target!(|data: &[u8]| {
    let _ = String::from_shift_jis_2004(data);
});
