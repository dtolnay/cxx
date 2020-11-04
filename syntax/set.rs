use std::fmt::{self, Debug};
use std::slice;

pub use self::ordered::OrderedSet;
pub use self::unordered::UnorderedSet;

mod ordered {
    use super::{Iter, UnorderedSet};
    use std::borrow::Borrow;
    use std::hash::Hash;

    pub struct OrderedSet<T> {
        set: UnorderedSet<T>,
        vec: Vec<T>,
    }

    impl<'a, T> OrderedSet<&'a T>
    where
        T: Hash + Eq,
    {
        pub fn new() -> Self {
            OrderedSet {
                set: UnorderedSet::new(),
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
}

mod unordered {
    use std::borrow::Borrow;
    use std::collections::HashSet;
    use std::hash::Hash;

    // Wrapper prohibits accidentally introducing iteration over the set, which
    // could lead to nondeterministic generated code.
    pub struct UnorderedSet<T>(HashSet<T>);

    impl<T> UnorderedSet<T>
    where
        T: Hash + Eq,
    {
        pub fn new() -> Self {
            UnorderedSet(HashSet::new())
        }

        pub fn insert(&mut self, value: T) -> bool {
            self.0.insert(value)
        }

        pub fn contains<Q>(&self, value: &Q) -> bool
        where
            T: Borrow<Q>,
            Q: ?Sized + Hash + Eq,
        {
            self.0.contains(value)
        }

        pub fn get<Q>(&self, value: &Q) -> Option<&T>
        where
            T: Borrow<Q>,
            Q: ?Sized + Hash + Eq,
        {
            self.0.get(value)
        }
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
