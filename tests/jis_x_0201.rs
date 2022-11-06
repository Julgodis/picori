#[cfg(test)]
mod jis_x_0201 {
    use picori::string::jis_x_0201::{JisX0201, JisX0201Iterator};

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

        assert!(&[0x5c].iter().jisx0201().all(|x| match x {
            Ok(c) => c == '\u{00a5}',
            Err(_) => true,
        }));

        assert!(&[0x7e].iter().jisx0201().all(|x| match x {
            Ok(c) => c == '\u{203e}',
            Err(_) => true,
        }));
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
    fn until_zero() {
        let data = b"abc\0def";
        let first = JisX0201::first(data).unwrap();
        let second = JisX0201::first(&data[4..]).unwrap();

        assert_eq!(first, "abc");
        assert_eq!(second, "def");
    }
}
