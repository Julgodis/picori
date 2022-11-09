#[cfg(test)]
mod shift_jis_1997 {
    use picori::{ShiftJis1997, ShiftJis1997IteratorExt};

    static TEST_UTF8: &[u8] = include_bytes!("../assets/tests/shift-jis/1997.ok.utf-8.txt");
    static TEST_SHIFTJIS: &[u8] = include_bytes!("../assets/tests/shift-jis/1997.ok.shift-jis.txt");
    static TEST_ERROR_2BYTE_SHIFTJIS: &[u8] =
        include_bytes!("../assets/tests/shift-jis/1997.error.two-byte.shift-jis.txt");

    #[test]
    fn ok() {
        let utf8 = String::from_utf8(TEST_UTF8.to_vec()).unwrap();
        let shift_jis = ShiftJis1997::all(TEST_SHIFTJIS).unwrap();

        assert_eq!(utf8.len(), shift_jis.len());
        assert_eq!(utf8, shift_jis);
    }

    #[test]
    fn err_two_byte() {
        for x in TEST_ERROR_2BYTE_SHIFTJIS.chunks(2) {
            let result = ShiftJis1997::all(x);
            assert!(result.is_err());
        }
    }

    #[test]
    fn first() {
        let data = b"\x83\x5b\x83\x8b\x83\x5f\x83\x93\x82\xcc\x93\x60\x90\xe0\0\x95\x97\x82\xcc\x83\x5e\x83\x4e\x83\x67";
        let first = ShiftJis1997::first(data).unwrap();
        let second = data
            .iter()
            .skip_while(|x| **x != 0)
            .skip(1)
            .sjis1997()
            .collect::<Result<String, _>>()
            .unwrap();

        assert_eq!(first, "ゼルダンの伝説");
        assert_eq!(second, "風のタクト");
        assert!(&ShiftJis1997::first(b"\xff").is_err());
    }

    #[test]
    fn iter() {
        let data = b"abcdef";
        assert_eq!(
            ShiftJis1997::iter(data.iter())
                .map(|x| x.unwrap())
                .collect::<String>(),
            "abcdef"
        );
    }

    #[test]
    fn all() {
        let data = b"abc\0def";
        assert_eq!(&ShiftJis1997::all(data).unwrap()[..], "abc\0def");
    }
}
