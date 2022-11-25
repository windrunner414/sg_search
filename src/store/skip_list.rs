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

const MAX_LEVEL: u8 = 32;
const LEVEL_P: f64 = 0.25;

#[inline]
fn random_level() -> u8 {
    let mut level = 1u8;
    while rand::random::<u16>() < (LEVEL_P * (u16::MAX as f64)) as u16 {
        level += 1;
    }
    cmp::min(level, MAX_LEVEL)
}

#[derive(Debug, Encode, Decode)]
struct MetaData {
    heads: [Level; MAX_LEVEL as usize],
    len: u64,
    level: u8,
}

impl MetaData {
    #[inline]
    fn empty() -> Self {
        Self {
            heads: [Level::empty(); MAX_LEVEL as usize],
            len: 0,
            level: 0,
        }
    }
}

#[derive(Debug, Encode, Decode, Copy, Clone)]
struct Level {
    backward: u64,
    forward: u64,
}

impl Level {
    #[inline]
    fn empty() -> Self {
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
struct Node<T: Encode + Decode> {
    data: T,
    /// We only support appending nodes
    /// once a node is created, the number of level won't change
    /// so we don't need an array of length SKIP_LIST_MAX_LEVEL
    /// We have to initialize this vector so that its length is equal to the number of level of the current node
    /// because we cannot change the size of the space it occupies
    levels: Vec<Level>,
}

#[derive(Debug)]
pub struct Reader<'a, S: ReadableStore, T: Encode + Decode> {
    store: &'a S,
    offset: u64,
    _marker: PhantomData<T>,
}

impl<'a, S: ReadableStore, T: Encode + Decode> Reader<'a, S, T> {
    #[inline]
    pub fn new(store: &'a S, offset: u64) -> Self {
        Self {
            store,
            offset,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    #[inline]
    fn metadata(&self) -> Result<MetaData> {
        self.store.read(self.offset, BIN_CODE_CONFIG)
    }

    #[inline]
    fn node(&self, offset: u64) -> Result<Node<T>> {
        self.store.read(offset, BIN_CODE_CONFIG)
    }

    #[inline]
    pub fn iter(&self) -> Result<Iter<'_, S, T>> {
        Iter::new(self)
    }

    /// We may use a type different from data (T) to do the search
    /// So we receive a cmp function instead of data
    #[inline]
    pub fn find<F: Fn(&T) -> Ordering>(&self, cmp: F) -> Result<Option<T>> {
        let metadata = self.metadata()?;
        let mut _levels_vec = None;
        let mut levels = metadata.heads.as_slice();

        for i in (0..metadata.level as usize).rev() {
            loop {
                let forward_offset = unsafe { levels.get_unchecked(i).forward };
                if forward_offset == 0 {
                    break;
                }
                let forward = self.node(forward_offset)?;
                match cmp(&forward.data) {
                    Ordering::Less => levels = _levels_vec.insert(forward.levels).as_slice(),
                    Ordering::Equal => return Ok(Some(forward.data)),
                    Ordering::Greater => break,
                }
            }
        }

        Ok(None)
    }
}

pub struct Iter<'a, S: ReadableStore, T: Encode + Decode> {
    reader: &'a Reader<'a, S, T>,
    next: u64,
}

impl<'a, S: ReadableStore, T: Encode + Decode> Iter<'a, S, T> {
    #[inline]
    fn new(reader: &'a Reader<'a, S, T>) -> Result<Self> {
        let metadata = reader.metadata()?;
        Ok(Self {
            reader,
            next: metadata.heads[0].forward,
        })
    }
}

impl<'a, S: ReadableStore, T: Encode + Decode> Iterator for Iter<'a, S, T> {
    type Item = Result<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next == 0 {
            None
        } else {
            Some(self.reader.node(self.next).map(|node| {
                self.next = unsafe { node.levels.get_unchecked(0).forward };
                node.data
            }))
        }
    }
}

#[derive(Debug)]
pub struct Writer<'a, S: ReadableStore + WriteableStore, T: Encode + Decode> {
    store: &'a mut S,
    offset: u64,
    phantom: PhantomData<T>,
}

impl<'a, S: ReadableStore + WriteableStore, T: Encode + Decode> Writer<'a, S, T> {
    #[inline]
    pub fn new(store: &'a mut S, offset: u64) -> Self {
        Self {
            store,
            offset,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn make_append(store: &'a mut S) -> Result<Self> {
        let offset = store.append(MetaData::empty(), BIN_CODE_CONFIG)?;
        Ok(Self::new(store, offset))
    }

    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    #[inline]
    fn metadata(&self) -> Result<MetaData> {
        self.store.read(self.offset, BIN_CODE_CONFIG)
    }

    #[inline]
    fn node(&self, offset: u64) -> Result<Node<T>> {
        self.store.read(offset, BIN_CODE_CONFIG)
    }

    pub fn push_back(&mut self, data: T) -> Result<()> {
        let level = random_level();
        let mut metadata = self.metadata()?;
        let mut node = Node {
            data,
            levels: vec![Level::empty(); level as usize],
        };
        for (heads, level) in metadata.heads.iter().zip(node.levels.iter_mut()) {
            level.backward = heads.backward;
        }
        let offset = self.store.append(node, BIN_CODE_CONFIG)?;
        for i in 0..level as usize {
            let heads = unsafe { metadata.heads.get_unchecked_mut(i) };
            if heads.backward == 0 {
                heads.forward = offset;
                heads.backward = offset;
            } else {
                let mut back_node = self.node(heads.backward)?;
                unsafe {
                    back_node.levels.get_unchecked_mut(i).forward = offset;
                }
                self.store
                    .write(&back_node, heads.backward, BIN_CODE_CONFIG)?;
                heads.backward = offset;
            }
        }
        metadata.len += 1;
        metadata.level = cmp::max(metadata.level, level);
        self.store.write(&metadata, self.offset, BIN_CODE_CONFIG)?;

        Ok(())
    }

    #[inline]
    pub fn into_reader(self) -> Reader<'a, S, T> {
        Reader::new(self.store, self.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{FileStore, MMapStore};
    use fake::Fake;
    use std::collections::HashSet;
    use std::fs;
    use std::fs::OpenOptions;
    use std::path::Path;

    const TEST_FILE_PATH: &str = "./test_output/skip_list1.test";

    fn clear_file() {
        let path = Path::new(TEST_FILE_PATH);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .unwrap();
        file.set_len(0).unwrap();
    }

    #[test]
    fn skip_list_1() {
        for _ in 0..5 {
            clear_file();
            let path = Path::new(TEST_FILE_PATH);

            let mut file_store = FileStore::open(path).unwrap();
            file_store.write_bytes(&[1], 0).unwrap();
            let mut writer = Writer::make_append(&mut file_store).unwrap();

            let mut n = (-100..100).fake::<i64>();
            let mut data = HashSet::new();
            for i in 0..(1000..5000).fake() {
                n += (1..10).fake::<i64>();
                writer.push_back(n).unwrap();
                data.insert(n);
            }

            let mmap_store = MMapStore::open(path).unwrap();
            let reader: Reader<MMapStore, i64> = Reader::new(&mmap_store, 1);
            let metadata = reader.metadata().unwrap();

            assert_eq!(metadata.len, data.len() as u64);

            let last = i64::MIN;
            for i in reader.iter().unwrap() {
                let curr = i.unwrap();
                assert!(curr > last);
                assert!(data.contains(&curr));
            }

            for i in 0..5000 {
                let f = (-150..n).fake::<i64>();
                let result = reader.find(|i| i.cmp(&f)).unwrap();
                if data.contains(&f) {
                    assert_eq!(result, Some(f));
                } else {
                    assert_eq!(result, None);
                }
            }
        }
    }
}
