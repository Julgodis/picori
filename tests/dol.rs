#[cfg(test)]
mod dol {
    use std::io::Cursor;

    use picori::dol::{Section, SectionKind};
    use picori::Dol;

    #[test]
    fn invalid_header_size() {
        let mut dol = Vec::new();
        dol.extend_from_slice(&[0; 1]);
        assert!(Dol::from_binary(&mut Cursor::new(dol)).is_err());
    }

    #[test]
    fn header() {
        let text_offset: [u32; 7] = [1, 2, 3, 4, 5, 6, 7];
        let text_address: [u32; 7] = [1, 2, 3, 4, 5, 6, 7];
        let text_size: [u32; 7] = [0, 0, 0, 0, 0, 0, 0];
        let data_offset: [u32; 11] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let data_address: [u32; 11] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let data_size: [u32; 11] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let mut dol = Vec::new();

        for offset in &text_offset {
            dol.extend_from_slice(&offset.to_be_bytes());
        }

        for offset in &data_offset {
            dol.extend_from_slice(&offset.to_be_bytes());
        }

        for address in &text_address {
            dol.extend_from_slice(&address.to_be_bytes());
        }

        for address in &data_address {
            dol.extend_from_slice(&address.to_be_bytes());
        }

        for size in &text_size {
            dol.extend_from_slice(&size.to_be_bytes());
        }

        for size in &data_size {
            dol.extend_from_slice(&size.to_be_bytes());
        }

        dol.extend_from_slice(&0x19876543_u32.to_be_bytes());
        dol.extend_from_slice(&0x29876543_u32.to_be_bytes());
        dol.extend_from_slice(&0x39876543_u32.to_be_bytes());

        dol.extend_from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);

        let dol = Dol::from_binary(&mut Cursor::new(dol)).unwrap();
        assert_eq!(dol.header.text_offset, text_offset);
        assert_eq!(dol.header.text_address, text_address);
        assert_eq!(dol.header.text_size, text_size);
        assert_eq!(dol.header.data_offset, data_offset);
        assert_eq!(dol.header.data_address, data_address);
        assert_eq!(dol.header.data_size, data_size);
        assert_eq!(dol.header.bss_address, 0x19876543);
        assert_eq!(dol.header.bss_size, 0x29876543);
        assert_eq!(dol.header.entry_point, 0x39876543);
        assert_eq!(dol.entry_point(), 0x39876543);
    }

    #[test]
    fn invalid_section_size() {
        let mut dol = Vec::new();
        dol.extend_from_slice(&[1; 0x90]);
        dol.extend_from_slice(&[0; 0x70]);
        let text0_size = 18 * 4 + 18 * 4;
        let text0_size_data = &[0xff, 0xff, 0xff, 0xff];
        dol.splice(text0_size..text0_size + 4, text0_size_data.iter().copied());
        assert!(Dol::from_binary(&mut Cursor::new(dol)).is_err());
    }

    #[test]
    #[should_panic]
    fn section_name_text_unreachable() { Section::guess_name(SectionKind::Text, 7); }

    #[test]
    #[should_panic]
    fn section_name_data_unreachable() { Section::guess_name(SectionKind::Data, 11); }

    #[test]
    #[should_panic]
    fn section_name_bss_unreachable() { Section::guess_name(SectionKind::Bss, 3); }

    #[test]
    fn read_zeroed_sections0() {
        let mut file = include_bytes!("../assets/tests/dol/test0.dol");
        let mut cursor = Cursor::new(&mut file);
        let dol = Dol::from_binary(&mut cursor).unwrap();

        let init = dol.section_by_name(".init").unwrap();
        assert_eq!(init.name, ".init");
        assert_eq!(init.kind, SectionKind::Text);
        assert_eq!(init.address, 0x8000_3100);
        assert_eq!(init.size, 0x2500);
        assert_eq!(init.aligned_size, 0x2500);
        assert_eq!(init.data, vec![0_u8; 0x2500]);

        let text = dol.section_by_name(".text").unwrap();
        assert_eq!(text.name, ".text");
        assert_eq!(text.kind, SectionKind::Text);
        assert_eq!(text.address, 0x8000_56C0);
        assert_eq!(text.size, 0x36_E100);
        assert_eq!(text.aligned_size, 0x36_E100);
        assert_eq!(text.data, vec![0_u8; 0x36_E100]);
    }

    #[test]
    fn read_zeroed_sections1() {
        let mut file = include_bytes!("../assets/tests/dol/test1.dol");
        let mut cursor = Cursor::new(&mut file);
        let dol = Dol::from_binary(&mut cursor).unwrap();
        assert!(dol.rom_copy_info.is_some());
        assert!(dol.bss_init_info.is_some());

        let init = dol.section_by_name(".init").unwrap();
        assert_eq!(init.name, ".init");
        assert_eq!(init.kind, SectionKind::Text);
        assert_eq!(init.address, 0x8000_3100);
        assert_eq!(init.size, 0x24E8);
        assert_eq!(init.aligned_size, 0x2500);

        let text = dol.section_by_name(".text").unwrap();
        assert_eq!(text.name, ".text");
        assert_eq!(text.kind, SectionKind::Text);
        assert_eq!(text.address, 0x8000_56C0);
        assert_eq!(text.size, 0x36_E0F4);
        assert_eq!(text.aligned_size, 0x36_E100);
        assert_eq!(text.data, vec![0_u8; 0x36_E100]);

        let init2 = dol.section_by_address(init.address).unwrap();
        assert_eq!(init2.name, ".init");
        assert_eq!(init2.kind, SectionKind::Text);
        assert_eq!(init2.address, 0x8000_3100);
        assert_eq!(init2.size, 0x24E8);
        assert_eq!(init2.aligned_size, 0x2500);
    }
}
