use std::collections::HashSet;
use std::hash::Hash;

pub struct UniqueCollection<T> {
    values: Vec<T>,
}

impl<T> UniqueCollection<T>
where
    T: Clone + Eq + Hash,
{
    pub fn new(values: Vec<T>) -> Self {
        let mut seen = HashSet::new();
        let unique_values = values
            .into_iter()
            .filter(|value| seen.insert(value.clone()))
            .collect::<Vec<_>>();

        Self {
            values: unique_values,
        }
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }
}
