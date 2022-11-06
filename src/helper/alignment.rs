use std::ops::{Add, BitAnd, Not, Sub};

pub trait AlignPowerOfTwo {
    fn align_next(self, alignment: u32) -> Self;
}

impl AlignPowerOfTwo for u32 {
    fn align_next(self, alignment: u32) -> Self {
        assert!(
            alignment.is_power_of_two(),
            "alignment must be a power of two"
        );
        let alignment: Self = (alignment - 1).into();
        self.wrapping_add(alignment) & !alignment
    }
}
