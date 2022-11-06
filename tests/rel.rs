#[cfg(test)]
mod rel {
    use std::io::Cursor;

    use picori::{rel, Rel};

    #[test]
    fn test0() {
        let data = include_bytes!("../assets/tests/rel/test0.rel");
        let rel = Rel::from_binary(Cursor::new(&data)).unwrap();
        assert_eq!(rel.module, 400);
        assert_eq!(rel.version, 3);
        assert_eq!(rel.name_offset, 0x4216);
        assert_eq!(rel.name_size, 0x28);
        assert_eq!(rel.alignment, 0x8);
        assert_eq!(rel.bss_alignment, 0x1);
        assert_eq!(rel.fix_size, 0x21c0);
        assert_eq!(rel.relocation_offset, Some(0x21c0));
        assert_eq!(rel.import_offset, Some(0x21b0));
        assert_eq!(rel.import_size, Some(0x10));
        assert_eq!(
            rel.prolog,
            Some(rel::Symbol {
                section: 1,
                offset:  0x00,
            })
        );
        assert_eq!(
            rel.epilog,
            Some(rel::Symbol {
                section: 1,
                offset:  0x2c,
            })
        );
        assert_eq!(
            rel.unresolved,
            Some(rel::Symbol {
                section: 1,
                offset:  0x58,
            })
        );

        let reloctions = rel.relocations().count();
        assert_eq!(reloctions, 450);
    }

    #[test]
    fn test0_v1() {
        let data = include_bytes!("../assets/tests/rel/test0_v1.rel");
        let rel = Rel::from_binary(Cursor::new(&data)).unwrap();
        assert_eq!(rel.version, 1);
    }

    #[test]
    fn test0_v2() {
        let data = include_bytes!("../assets/tests/rel/test0_v2.rel");
        let rel = Rel::from_binary(Cursor::new(&data)).unwrap();
        assert_eq!(rel.version, 2);
    }

    #[test]
    fn test0_v4() {
        let data = include_bytes!("../assets/tests/rel/test0_v4.rel");
        let rel = Rel::from_binary(Cursor::new(&data));
        assert!(rel.is_err());
    }

    #[test]
    fn test0_only_null_section() {
        let data = include_bytes!("../assets/tests/rel/test0_only_null_section.rel");
        let rel = Rel::from_binary(Cursor::new(&data));
        assert!(rel.is_err());
    }

    #[test]
    fn test0_section_offset() {
        let data = include_bytes!("../assets/tests/rel/test0_section_offset.rel");
        let rel = Rel::from_binary(Cursor::new(&data));
        assert!(rel.is_err());
    }

    #[test]
    fn test0_no_prolog() {
        let data = include_bytes!("../assets/tests/rel/test0_no_prolog.rel");
        let rel = Rel::from_binary(Cursor::new(&data)).unwrap();
        assert!(rel.prolog.is_none());
    }

    #[test]
    fn test0_relocation() {
        let data = include_bytes!("../assets/tests/rel/test0_relocation.rel");
        let rel = Rel::from_binary(Cursor::new(&data));
        assert!(rel.is_ok());

        let _ = rel.unwrap().relocations().count();
    }

    #[test]
    fn test0_unknown_import_kind() {
        let data = include_bytes!("../assets/tests/rel/test0_unknown_import_kind.rel");
        let rel = Rel::from_binary(Cursor::new(&data));
        assert!(rel.is_err());
    }

    #[test]
    fn test0_relocation_without_section() {
        let data = include_bytes!("../assets/tests/rel/test0_relocation_without_section.rel");
        let rel = Rel::from_binary(Cursor::new(&data));
        assert!(rel.is_ok());

        let _ = rel.unwrap().relocations().count();
    }

    #[test]
    fn test0_section_size() {
        let data = include_bytes!("../assets/tests/rel/test0_section_size.rel");
        let rel = Rel::from_binary(Cursor::new(&data));
        assert!(rel.is_err());
    }

    #[test]
    fn test1() {
        let data = include_bytes!("../assets/tests/rel/test1.rel");
        let rel = Rel::from_binary(Cursor::new(&data)).unwrap();
        assert_eq!(rel.module, 202);
        assert_eq!(rel.version, 3);
    }
}
