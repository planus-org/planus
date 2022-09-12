use core::{
    convert::TryInto,
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
};

use hashbrown::raw::RawTable;

pub(crate) trait Cacheable {
    fn lookup(serialized: &[u8]) -> Option<&[u8]>;
}

pub(crate) struct VTable;

impl Cacheable for VTable {
    fn lookup(serialized: &[u8]) -> Option<&[u8]> {
        let length = u16::from_le_bytes(serialized.get(..2)?.try_into().ok()?);
        serialized.get(..length as usize)
    }
}

pub(crate) struct ByteVec;

impl Cacheable for ByteVec {
    fn lookup(serialized: &[u8]) -> Option<&[u8]> {
        let length = u32::from_le_bytes(serialized.get(..4)?.try_into().ok()?);
        serialized.get(4..4 + length as usize)
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

fn hash_one<H: BuildHasher>(hash_builder: &H, value: &[u8]) -> u64 {
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
                serialized_data
                    .len()
                    .checked_sub((*back_offset).try_into().unwrap())
                    .and_then(|offset| C::lookup(&serialized_data[offset..]))
                    .map_or(false, |old_key| old_key == key)
            })
            .copied()
    }

    /// Should only be called if `get` returned `None`
    pub(crate) fn insert(&mut self, key_hash: u64, back_offset: u32, serialized_data: &[u8]) {
        self.cache.insert(key_hash, back_offset, |back_offset| {
            serialized_data
                .len()
                .checked_sub((*back_offset).try_into().unwrap())
                .and_then(|offset| C::lookup(&serialized_data[offset..]))
                .map_or(0, |old_key| hash_one(&self.hash_builder, old_key))
        });
    }

    pub(crate) fn clear(&mut self) {
        self.cache.clear_no_drop();
    }
}
