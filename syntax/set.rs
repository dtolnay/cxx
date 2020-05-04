use std::collections::HashSet;
use std::hash::Hash;
use std::slice;

pub struct OrderedSet<'a, T> {
    set: HashSet<&'a T>,
    vec: Vec<&'a T>,
}

impl<'a, T> OrderedSet<'a, T>
where
    T: Hash + Eq,
{
    pub fn new() -> Self {
        OrderedSet {
            set: HashSet::new(),
            vec: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: &'a T) -> bool {
        let new = self.set.insert(value);
        if new {
            self.vec.push(value);
        }
        new
    }

    pub fn contains(&self, value: &T) -> bool {
        self.set.contains(value)
    }
}

impl<'s, 'a, T> IntoIterator for &'s OrderedSet<'a, T> {
    type Item = &'a T;
    type IntoIter = Iter<'s, 'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        Iter(self.vec.iter())
    }
}

pub struct Iter<'s, 'a, T>(slice::Iter<'s, &'a T>);

impl<'s, 'a, T> Iterator for Iter<'s, 'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }
}
