use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::Index;

pub use self::unordered::UnorderedMap;
pub use std::collections::hash_map::Entry;

mod unordered {
    use crate::syntax::set::UnorderedSet;
    use std::borrow::Borrow;
    use std::collections::hash_map::{Entry, HashMap};
    use std::hash::Hash;

    // Wrapper prohibits accidentally introducing iteration over the map, which
    // could lead to nondeterministic generated code.
    pub struct UnorderedMap<K, V>(HashMap<K, V>);

    impl<K, V> UnorderedMap<K, V> {
        pub fn new() -> Self {
            UnorderedMap(HashMap::new())
        }
    }

    impl<K, V> UnorderedMap<K, V>
    where
        K: Hash + Eq,
    {
        pub fn insert(&mut self, key: K, value: V) -> Option<V> {
            self.0.insert(key, value)
        }

        pub fn contains_key<Q>(&self, key: &Q) -> bool
        where
            K: Borrow<Q>,
            Q: ?Sized + Hash + Eq,
        {
            self.0.contains_key(key)
        }

        pub fn get<Q>(&self, key: &Q) -> Option<&V>
        where
            K: Borrow<Q>,
            Q: ?Sized + Hash + Eq,
        {
            self.0.get(key)
        }

        pub fn entry(&mut self, key: K) -> Entry<K, V> {
            self.0.entry(key)
        }

        pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
        where
            K: Borrow<Q>,
            Q: ?Sized + Hash + Eq,
        {
            self.0.remove(key)
        }

        pub fn keys(&self) -> UnorderedSet<K>
        where
            K: Copy,
        {
            let mut set = UnorderedSet::new();
            for key in self.0.keys() {
                set.insert(*key);
            }
            set
        }
    }
}

impl<K, V> Default for UnorderedMap<K, V> {
    fn default() -> Self {
        UnorderedMap::new()
    }
}

impl<Q, K, V> Index<&Q> for UnorderedMap<K, V>
where
    K: Borrow<Q> + Hash + Eq,
    Q: ?Sized + Hash + Eq,
{
    type Output = V;

    fn index(&self, key: &Q) -> &V {
        self.get(key).unwrap()
    }
}
