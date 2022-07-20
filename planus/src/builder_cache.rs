use core::{
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
};

use hashbrown::raw::RawTable;

pub(crate) trait Cacheable {
    fn slice_matches(serialized: &[u8], key: &[u8]) -> bool;
}

pub(crate) struct VTable;

impl Cacheable for VTable {
    fn slice_matches(serialized: &[u8], key: &[u8]) -> bool {
        serialized.starts_with(key)
    }
}

pub(crate) struct ByteVec;

impl Cacheable for ByteVec {
    fn slice_matches(serialized: &[u8], key: &[u8]) -> bool {
        serialized.get(..4).map_or(false, |serialized| {
            serialized == (key.len() as u32).to_le_bytes()
        }) && serialized
            .get(4..)
            .map_or(false, |serialized| serialized.starts_with(key))
    }
}

pub(crate) struct Cache<C> {
    _marker: PhantomData<C>,
    cache: RawTable<u32>,
    hash_builder: hashbrown::hash_map::DefaultHashBuilder,
}

impl<C> Default for Cache<C> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
            cache: Default::default(),
            hash_builder: Default::default(),
        }
    }
}

impl<C> core::fmt::Debug for Cache<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Cache<{}>", core::any::type_name::<C>())
    }
}

fn hash_one<H: BuildHasher, T: Hash>(hash_builder: &H, value: T) -> u64 {
    let mut hasher = hash_builder.build_hasher();
    value.hash(&mut hasher);
    hasher.finish()
}

impl<C: Cacheable> Cache<C> {
    pub(crate) fn hash(&self, serialized_data: &[u8]) -> u64 {
        hash_one(&self.hash_builder, serialized_data)
    }

    pub(crate) fn get(&mut self, serialized_data: &[u8], key_hash: u64, key: &[u8]) -> Option<u32> {
        self.cache
            .get(key_hash, |back_offset| {
                if let Some(offset) = serialized_data
                    .len()
                    .checked_sub((*back_offset).try_into().unwrap())
                {
                    C::slice_matches(&serialized_data[offset..], key)
                } else {
                    false
                }
            })
            .copied()
    }

    /// Should only be called if `get` returned `None`
    pub(crate) fn insert(&mut self, key_hash: u64, back_offset: u32) {
        self.cache
            .insert(key_hash, back_offset, |v| hash_one(&self.hash_builder, v));
    }

    pub(crate) fn clear(&mut self) {
        self.cache.clear_no_drop();
    }
}
