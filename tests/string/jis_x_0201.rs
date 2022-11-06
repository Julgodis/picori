#[cfg(test)]
mod tests {
    use picori::string::{JisX0201, JisX0201IteratorExt};

    #[test]
    fn ascii() {
        let result = (0..=0x7f)
            .filter(|x| *x != 0x5c)
            .filter(|x| *x != 0x7e)
            .jisx0201()
            .map(|x| x.unwrap());

        let ok = (0..=0x7f)
            .filter(|x| *x != 0x5c)
            .filter(|x| *x != 0x7e)
            .zip(result)
            .map(|(a, b)| (a as u8) as char == b)
            .all(|x| x);

        assert!(ok);
        assert!([0x5c].iter().jisx0201().next().unwrap().unwrap() == '\u{00a5}');
        assert!([0x7e].iter().jisx0201().next().unwrap().unwrap() == '\u{203e}');
    }

    #[test]
    fn halfwidth_katakana() {
        let result = (0xa1..=0xdf)
            .jisx0201()
            .map(|x| x.unwrap())
            .zip(0xa1..=0xdf)
            .all(|(x, i)| {
                let unicode = 0xFF61 + (i as u8 - 0xa1) as u32;
                char::from_u32(unicode).unwrap() == x
            });
        assert!(result);
    }

    #[test]
    fn err() {
        let result1 = (0x80..=0xa0).jisx0201().all(|x| x.is_err());
        let result2 = (0xe0..=0xff).jisx0201().all(|x| x.is_err());

        assert!(result1);
        assert!(result2);
    }

    #[test]
    fn first() {
        assert_eq!(&JisX0201::first(b"abc\0def").unwrap()[..], "abc");
        assert!(&JisX0201::first(b"abc\xa0def").is_err());
    }

    #[test]
    fn iter() {
        let data = b"abcdef";
        assert_eq!(
            JisX0201::iter(data.iter())
                .map(|x| x.unwrap())
                .collect::<String>(),
            "abcdef"
        );
    }

    #[test]
    fn all() {
        let data = b"abc\0def";
        assert_eq!(&JisX0201::all(data).unwrap()[..], "abc\0def");
        assert!(&JisX0201::all(b"abc\xa0def").is_err());
    }
}
