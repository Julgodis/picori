#[cfg(test)]
mod shift_ji_1997 {
    use picori::string::shift_jis::{ShiftJis1997, ShiftJis1997Iterator};

    static TEST_1997_UTF8: &[u8] = include_bytes!("../assets/tests/shift-jis/1997.ok.utf-8.txt");
    static TEST_1997_SHIFTJIS: &[u8] =
        include_bytes!("../assets/tests/shift-jis/1997.ok.shift-jis.txt");
    static TEST_ERROR_2BYTE_1997_SHIFTJIS: &[u8] =
        include_bytes!("../assets/tests/shift-jis/1997.error.two-byte.shift-jis.txt");

    #[test]
    fn ok() {
        let utf8 = String::from_utf8(TEST_1997_UTF8.to_vec()).unwrap();
        let shift_jis = ShiftJis1997::all(TEST_1997_SHIFTJIS).unwrap();

        assert_eq!(utf8.len(), shift_jis.len());
        assert_eq!(utf8, shift_jis);
    }

    #[test]
    fn err_two_byte() {
        for x in TEST_ERROR_2BYTE_1997_SHIFTJIS.chunks(2) {
            let result = ShiftJis1997::all(x);
            assert!(result.is_err());
        }
    }

    #[test]
    fn until_zero() {
        let data = b"\x83\x5b\x83\x8b\x83\x5f\x83\x93\x82\xcc\x93\x60\x90\xe0\0\x95\x97\x82\xcc\x83\x5e\x83\x4e\x83\x67";
        let first = ShiftJis1997::first(data).unwrap();
        let second = 
            data.iter().skip_while(|x| **x != 0).skip(1)
            .sjis1997().collect::<Result<String, _>>().unwrap();

        assert_eq!(first, "ゼルダンの伝説");
        assert_eq!(second, "風のタクト");
    }
}
