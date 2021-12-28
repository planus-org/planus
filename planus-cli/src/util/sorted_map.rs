#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct SortedMap<K, V>(pub Vec<(K, V)>);

pub struct SortedSet<K>(SortedMap<K, ()>);

impl<K, V> Default for SortedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> Default for SortedSet<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> SortedMap<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.0.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().map(|(_, v)| v)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.0.iter_mut().map(|(_, v)| v)
    }

    pub fn first(&self) -> Option<&(K, V)> {
        self.0.first()
    }

    pub fn first_value(&self) -> Option<&V> {
        self.first().map(|(_k, v)| v)
    }

    pub fn last(&self) -> Option<&(K, V)> {
        self.0.last()
    }

    pub fn last_value(&self) -> Option<&V> {
        self.last().map(|(_k, v)| v)
    }
}

impl<K: Ord, V> SortedMap<K, V> {
    pub fn index_of(&self, k: &K) -> Option<usize> {
        self.0.binary_search_by_key(&k, |(k, _v)| k).ok()
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        let index = self.0.binary_search_by_key(&k, |(k, _v)| k).ok()?;
        Some(&self.0[index].1)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.0.binary_search_by_key(&&key, |(k, _v)| k) {
            Ok(index) => Some(std::mem::replace(&mut self.0[index].1, value)),
            Err(index) => {
                self.0.insert(index, (key, value));
                None
            }
        }
    }

    pub fn entry(&mut self, key: K) -> sorted_map::Entry<'_, K, V> {
        match self.0.binary_search_by_key(&&key, |(k, _v)| k) {
            Ok(index) => {
                sorted_map::Entry::Occupied(sorted_map::OccupiedEntry { map: self, index })
            }
            Err(index) => sorted_map::Entry::Vacant(sorted_map::VacantEntry {
                map: self,
                key,
                index,
            }),
        }
    }
}

#[allow(clippy::module_inception)]
pub mod sorted_map {
    pub enum Entry<'a, K: 'a, V: 'a> {
        Occupied(OccupiedEntry<'a, K, V>),
        Vacant(VacantEntry<'a, K, V>),
    }

    pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
        pub(super) map: &'a mut super::SortedMap<K, V>,
        pub(super) index: usize,
    }

    pub struct VacantEntry<'a, K: 'a, V: 'a> {
        pub(super) map: &'a mut super::SortedMap<K, V>,
        pub(super) key: K,
        pub(super) index: usize,
    }

    impl<'a, K: 'a, V: 'a> Entry<'a, K, V> {
        /// Ensures a value is in the entry by inserting the default if empty, and returns
        /// a mutable reference to the value in the entry.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::collections::HashMap;
        ///
        /// let mut map: HashMap<&str, u32> = HashMap::new();
        ///
        /// map.entry("poneyland").or_insert(3);
        /// assert_eq!(map["poneyland"], 3);
        ///
        /// *map.entry("poneyland").or_insert(10) *= 2;
        /// assert_eq!(map["poneyland"], 6);
        /// ```
        #[inline]
        pub fn or_insert(self, default: V) -> &'a mut V {
            match self {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(default),
            }
        }

        /// Ensures a value is in the entry by inserting the result of the default function if empty,
        /// and returns a mutable reference to the value in the entry.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::collections::HashMap;
        ///
        /// let mut map: HashMap<&str, String> = HashMap::new();
        /// let s = "hoho".to_string();
        ///
        /// map.entry("poneyland").or_insert_with(|| s);
        ///
        /// assert_eq!(map["poneyland"], "hoho".to_string());
        /// ```
        #[inline]
        pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
            match self {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(default()),
            }
        }
    }

    impl<'a, K, V: Default> Entry<'a, K, V> {
        /// Ensures a value is in the entry by inserting the default value if empty,
        /// and returns a mutable reference to the value in the entry.
        ///
        /// # Examples
        ///
        /// ```
        /// # fn main() {
        /// use std::collections::HashMap;
        ///
        /// let mut map: HashMap<&str, Option<u32>> = HashMap::new();
        /// map.entry("poneyland").or_default();
        ///
        /// assert_eq!(map["poneyland"], None);
        /// # }
        /// ```
        #[inline]
        pub fn or_default(self) -> &'a mut V {
            match self {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(Default::default()),
            }
        }
    }

    impl<'a, K: 'a, V: 'a> OccupiedEntry<'a, K, V> {
        pub fn into_mut(self) -> &'a mut V {
            &mut self.map.0[self.index].1
        }

        pub fn get_mut(&mut self) -> &mut V {
            &mut self.map.0[self.index].1
        }
    }

    impl<'a, K: 'a, V: 'a> VacantEntry<'a, K, V> {
        pub fn insert(self, value: V) -> &'a mut V {
            self.map.0.insert(self.index, (self.key, value));
            &mut self.map.0[self.index].1
        }
    }
}

impl<K> SortedSet<K> {
    pub fn new() -> Self {
        Self(SortedMap::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn iter(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|(k, ())| k)
    }

    pub fn first(&self) -> Option<&K> {
        self.0.first().map(|(k, ())| k)
    }

    pub fn last(&self) -> Option<&K> {
        self.0.last().map(|(k, ())| k)
    }
}

impl<K: Ord> SortedSet<K> {
    pub fn index_of(&self, k: &K) -> Option<usize> {
        self.0.index_of(k)
    }

    pub fn insert(&mut self, key: K) -> bool {
        self.0.insert(key, ()).is_none()
    }

    pub fn contains(&self, k: &K) -> bool {
        self.0.get(k).is_some()
    }
}
