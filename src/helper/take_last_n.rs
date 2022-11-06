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
