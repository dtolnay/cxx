use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::{self, Debug};
use std::hash::Hash;
use std::slice;

pub struct OrderedSet<T> {
    set: HashSet<T>,
    vec: Vec<T>,
}

impl<'a, T> OrderedSet<&'a T>
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

    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        &'a T: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.set.contains(value)
    }

    pub fn get<Q>(&self, value: &Q) -> Option<&'a T>
    where
        &'a T: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.set.get(value).copied()
    }
}

impl<'s, 'a, T> IntoIterator for &'s OrderedSet<&'a T> {
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

impl<'a, T> Debug for OrderedSet<&'a T>
where
    T: Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_set().entries(self).finish()
    }
}
