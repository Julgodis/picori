#[cfg(test)]
mod jis_x_0201 {
    use picori::string::jis_x_0201::JisX0210Decoder;
    use picori::string::StringDecoder;

    #[test]
    fn ascii() {
        for x in 0..=0x7f {
            if x == 0x5c || x == 0x7e {
                continue;
            }

            let result = JisX0210Decoder::decode_bytes(&[x]);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), format!("{}", x as char));
        }

        let x5c = JisX0210Decoder::decode_bytes(&[0x5c]);
        assert!(x5c.is_ok());
        assert_eq!(x5c.unwrap(), "\u{00a5}".to_string());

        let x7e = JisX0210Decoder::decode_bytes(&[0x7e]);
        assert!(x7e.is_ok());
        assert_eq!(x7e.unwrap(), "\u{203e}".to_string());
    }

    #[test]
    fn halfwidth_katakana() {
        for x in 0xa1..=0xdf {
            let result = JisX0210Decoder::decode_bytes(&[x]);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                format!("{}", char::from_u32(0xFF61 + (x - 0xa1) as u32).unwrap())
            );
        }
    }

    #[test]
    fn err() {
        for x in 0x80..=0xa0 {
            let result = JisX0210Decoder::decode_bytes(&[x]);
            assert!(result.is_err());
        }

        for x in 0xE0..=0xff {
            let result = JisX0210Decoder::decode_bytes(&[x]);
            assert!(result.is_err());
        }
    }

    #[test]
    fn until_zero() {
        let data = b"abc\0def";
        let first = JisX0210Decoder::decode_until_zero(data).unwrap();
        let second = JisX0210Decoder::decode_until_zero_iterator(
            data.iter().skip_while(|x| **x != 0).skip(1).map(|x| *x),
        )
        .unwrap();

        assert_eq!(first, "abc");
        assert_eq!(second, "def");
    }
}
