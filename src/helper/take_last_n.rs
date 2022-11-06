pub trait TakeLastN<T> {
    fn take_last_n(&self, n: usize) -> &[T];
}

impl<T> TakeLastN<T> for &[T] {
    fn take_last_n(&self, n: usize) -> &[T] {
        if self.len() < n {
            &self[..]
        } else {
            &self[self.len() - n..]
        }
    }
}

// -------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        let v = Vec::<i32>::new();
        assert!((&v[..]).take_last_n(0).is_empty());
        let v = vec![1, 2, 3];
        assert!((&v[..]).take_last_n(0).is_empty());
    }

    #[test]
    fn n_under_m() {
        let v = vec![1, 2, 3];
        assert_eq!((&v[..]).take_last_n(4), &[1, 2, 3]);
    }

    #[test]
    fn n_equals_m() {
        let v = vec![1, 2, 3];
        assert_eq!((&v[..]).take_last_n(3), &[1, 2, 3]);
    }

    #[test]
    fn n_over_m() {
        let v = vec![1, 2, 3];
        assert_eq!((&v[..]).take_last_n(2), &[2, 3]);
    }
}
