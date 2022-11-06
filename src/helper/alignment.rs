pub trait AlignPowerOfTwo {
    fn align_next(self, alignment: u32) -> Self;
}

impl AlignPowerOfTwo for u32 {
    fn align_next(self, alignment: u32) -> Self {
        assert!(
            alignment.is_power_of_two(),
            "alignment must be a power of two"
        );
        let alignment = alignment - 1;
        self.wrapping_add(alignment) & !alignment
    }
}

// -------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_32() {
        assert_eq!(0u32.align_next(32), 0);
        assert_eq!(31u32.align_next(32), 32);
        assert_eq!(32u32.align_next(32), 32);
    }

    #[test]
    #[should_panic]
    fn align_0() { 0u32.align_next(0); }

    #[test]
    #[should_panic]
    fn align_non_power_of_two() { 0u32.align_next(3); }
}
