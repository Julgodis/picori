#[cfg(test)]
mod tests {
    use picori::encoding::{Ascii, AsciiIteratorExt};

    #[test]
    fn ok() {
        let result = (0..=0x7f).ascii().collect::<Result<String, _>>();
        assert!(result.is_ok());

        let ok = (0..=0x7f)
            .zip(result.unwrap().chars())
            .map(|(a, b)| a as u8 as char == b)
            .all(|x| x);
        assert!(ok);
    }

    #[test]
    fn err() {
        let result = (0x80..=0xff).ascii().all(|x| x.is_err());
        assert!(result);
    }

    #[test]
    fn first() {
        assert_eq!(&Ascii::first(b"abc\0def").unwrap()[..], "abc");
        assert!(&Ascii::first(b"abc\xff def").is_err());
    }

    #[test]
    fn iter() {
        let data = b"abcdef";
        assert_eq!(
            Ascii::iter(data.iter())
                .map(|x| x.unwrap())
                .collect::<String>(),
            "abcdef"
        );
    }

    #[test]
    fn all() {
        let data = b"abc\0def";
        assert_eq!(&Ascii::all(data).unwrap()[..], "abc\0def");
    }
}
