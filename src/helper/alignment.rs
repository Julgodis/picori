use std::ops::{Add, BitAnd, Not, Sub};

pub trait AlignPowerOfTwo
where
    Self: Sized
        + Copy
        + Add<Output = Self>
        + Sub<Output = Self>
        + BitAnd<Output = Self>
        + Not<Output = Self>
        + From<u32>,
{
    fn align_next(self, alignment: u32) -> Self {
        assert!(
            alignment.is_power_of_two(),
            "alignment must be a power of two"
        );
        let alignment: Self = (alignment - 1).into();
        (self + alignment) & !alignment
    }
}

impl AlignPowerOfTwo for u32 {}
