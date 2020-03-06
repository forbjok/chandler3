pub trait IsSubset<T> {
    fn is_subset(&self, other: &Vec<T>) -> bool;
}

impl<T: ToString + PartialEq> IsSubset<T> for Vec<T> {
    fn is_subset(&self, other: &Vec<T>) -> bool {
        self.iter().all(|x| other.contains(x))
    }
}
