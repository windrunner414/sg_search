use crate::store::Result;
use crate::store::{ReadableStore, WriteableStore};
use bincode::{Decode, Encode};
use std::cmp;
use std::cmp::Ordering;
use std::marker::PhantomData;

/// Skip list based on StoreIO
/// An valid offset must > 0, 0 is used as nullptr, we MUST place some data before the skip list

const BIN_CODE_CONFIG: bincode::config::Configuration<
    bincode::config::LittleEndian,
    bincode::config::Fixint,
    bincode::config::SkipFixedArrayLength,
> = bincode::config::standard()
    .with_little_endian()
    .with_fixed_int_encoding()
    .skip_fixed_array_length();

const SKIP_LIST_MAX_LEVEL: u8 = 32;
const SKIP_LIST_P: f64 = 0.25;

fn random_level() -> u8 {
    let mut level = 1u8;
    while rand::random::<u16>() < (SKIP_LIST_P * (u16::MAX as f64)) as u16 {
        level += 1;
    }
    cmp::min(level, SKIP_LIST_MAX_LEVEL)
}

#[derive(Debug, Encode, Decode)]
pub struct SkipListMetaData {
    heads: [SkipListLevel; SKIP_LIST_MAX_LEVEL as usize],
    len: u64,
    level: u8,
}

impl SkipListMetaData {
    pub fn empty() -> Self {
        Self {
            heads: [SkipListLevel::empty(); SKIP_LIST_MAX_LEVEL as usize],
            len: 0,
            level: 0,
        }
    }
}

#[derive(Debug, Encode, Decode, Copy, Clone)]
pub struct SkipListLevel {
    backward: u64,
    forward: u64,
}

impl SkipListLevel {
    pub fn empty() -> Self {
        Self {
            backward: 0,
            forward: 0,
        }
    }
}

// TODO: optimize
// we may not need the full data in "find()"
// so we can split it into "metadata" and "data", and only look for "metadata" in "find()"
// if we must use the full data, we could put real data into "metadata", and let "data" be a () type
#[derive(Debug, Encode, Decode)]
pub struct SkipListNode<T: Encode + Decode> {
    data: T,
    /// We only support appending nodes
    /// once a node is created, the number of level won't change
    /// so we don't need an array of length SKIP_LIST_MAX_LEVEL
    /// We have to initialize this vector so that its length is equal to the number of level of the current node
    /// because we cannot change the size of the space it occupies
    levels: Vec<SkipListLevel>,
}

#[derive(Debug)]
pub struct SkipListReader<'a, S: ReadableStore, T: Encode + Decode> {
    store: &'a S,
    offset: u64,
    phantom: PhantomData<T>,
}

impl<'a, S: ReadableStore, T: Encode + Decode> SkipListReader<'a, S, T> {
    pub fn new(store: &'a S, offset: u64) -> Self {
        Self {
            store,
            offset,
            phantom: PhantomData,
        }
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn metadata(&self) -> Result<SkipListMetaData> {
        self.store.read(self.offset, BIN_CODE_CONFIG)
    }

    /// We may use a type different from data (T) to do the search
    /// So we receive a cmp function instead of data
    pub fn find<F: Fn(&SkipListNode<T>) -> Ordering>(
        &self,
        cmp: F,
    ) -> Result<Option<SkipListNode<T>>> {
        let metadata = self.metadata()?;
        let mut _levels_vec = None;
        let mut levels = metadata.heads.as_slice();

        for i in (0..metadata.level as usize).rev() {
            loop {
                let forward_offset = unsafe { levels.get_unchecked(i).forward };
                if forward_offset == 0 {
                    break;
                }
                let forward = self.store.read(forward_offset, BIN_CODE_CONFIG)?;
                match cmp(&forward) {
                    Ordering::Less => levels = _levels_vec.insert(forward.levels).as_slice(),
                    Ordering::Equal => return Ok(Some(forward)),
                    Ordering::Greater => break,
                }
            }
        }

        Ok(None)
    }
}

#[derive(Debug)]
pub struct SkipListWriter<'a, S: ReadableStore + WriteableStore, T: Encode + Decode> {
    store: &'a mut S,
    offset: u64,
    phantom: PhantomData<T>,
}

impl<'a, S: ReadableStore + WriteableStore, T: Encode + Decode> SkipListWriter<'a, S, T> {
    pub fn new(store: &'a mut S, offset: u64) -> Self {
        Self {
            store,
            offset,
            phantom: PhantomData,
        }
    }

    pub fn make_append(store: &'a mut S) -> Result<Self> {
        let offset = store.append(SkipListMetaData::empty(), BIN_CODE_CONFIG)?;
        Ok(Self::new(store, offset))
    }

    pub fn into_reader(self) -> SkipListReader<'a, S, T> {
        SkipListReader::new(self.store, self.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {}
}
