pub trait IsSubset<T> {
    fn is_subset(&self, other: &[T]) -> bool;
}

impl<T: ToString + PartialEq> IsSubset<T> for &[T] {
    fn is_subset(&self, other: &[T]) -> bool {
        self.iter().all(|x| other.contains(x))
    }
}
