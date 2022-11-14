use crate::store::{Error, Result};
use bincode::config::Config as BinCodeConfig;
use bincode::{decode_from_slice, decode_from_std_read, encode_into_std_write, Decode, Encode};
use memmap2::{Mmap, MmapOptions};
use std::borrow::Cow;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Although read may modify some internal state (e.g. "file.seek()")
/// the change is not actually perceived externally
/// so we only need &self not &mut self
pub trait ReadableStore {
    fn read_bytes(&self, offset: u64, size: u64) -> Result<Cow<[u8]>>;

    fn read<D: Decode, C: BinCodeConfig>(&self, offset: u64, config: C) -> Result<D>;
}

pub trait WriteableStore {
    fn write_bytes(&mut self, buf: &[u8], offset: u64) -> Result<()>;

    fn append_bytes(&mut self, buf: &[u8]) -> Result<u64>;

    fn write<E: Encode, C: BinCodeConfig>(&mut self, data: E, offset: u64, config: C)
        -> Result<()>;

    fn append<E: Encode, C: BinCodeConfig>(&mut self, data: E, config: C) -> Result<u64>;
}

#[derive(Debug)]
pub struct FileStore {
    file: File,
}

impl FileStore {
    #[inline]
    pub fn open<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        Ok(FileStore {
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(file_path)?,
        })
    }
}

impl ReadableStore for FileStore {
    #[inline]
    fn read_bytes(&self, offset: u64, size: u64) -> Result<Cow<[u8]>> {
        let size: usize = size.try_into().map_err(|_| Error::OutOfRange)?;

        (&self.file).seek(SeekFrom::Start(offset))?;
        // notice: read to an uninitialized buffer is an undefined behavior
        let mut buf = vec![0u8; size];
        (&self.file).read_exact(&mut buf)?;

        Ok(Cow::Owned(buf))
    }

    #[inline]
    fn read<D: Decode, C: BinCodeConfig>(&self, offset: u64, config: C) -> Result<D> {
        (&self.file).seek(SeekFrom::Start(offset))?;
        // TODO: reuse buffer
        // it's safe to use a BufReader because we don't expect the data to be modified while being read
        let mut reader = BufReader::new(&self.file);
        Ok(decode_from_std_read(&mut reader, config)?)
    }
}

impl WriteableStore for FileStore {
    #[inline]
    fn write_bytes(&mut self, buf: &[u8], offset: u64) -> Result<()> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(buf)?;
        Ok(())
    }

    #[inline]
    fn append_bytes(&mut self, buf: &[u8]) -> Result<u64> {
        let offset = self.file.metadata()?.len();
        self.write_bytes(buf, offset)?;
        Ok(offset)
    }

    #[inline]
    fn write<E: Encode, C: BinCodeConfig>(
        &mut self,
        data: E,
        offset: u64,
        config: C,
    ) -> Result<()> {
        self.file.seek(SeekFrom::Start(offset))?;
        let mut writer = BufWriter::new(&self.file);
        encode_into_std_write(data, &mut writer, config)?;
        Ok(())
    }

    #[inline]
    fn append<E: Encode, C: BinCodeConfig>(&mut self, data: E, config: C) -> Result<u64> {
        let offset = self.file.metadata()?.len();
        self.write(data, offset, config)?;
        Ok(offset)
    }
}

#[derive(Debug)]
pub struct MMapStore {
    mmap: Mmap,
}

impl MMapStore {
    #[inline]
    pub fn open<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let file = File::open(file_path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(Self { mmap })
    }
}

impl ReadableStore for MMapStore {
    #[inline]
    fn read_bytes(&self, offset: u64, size: u64) -> Result<Cow<[u8]>> {
        let offset: usize = offset.try_into().map_err(|_| Error::OutOfRange)?;
        let size: usize = size.try_into().map_err(|_| Error::OutOfRange)?;

        if let Some(end) = offset.checked_add(size) {
            if self.mmap.len() >= end {
                Ok(Cow::Borrowed(&self.mmap[offset..end]))
            } else {
                Err(Error::OutOfRange)
            }
        } else {
            Err(Error::OutOfRange)
        }
    }

    #[inline]
    fn read<D: Decode, C: BinCodeConfig>(&self, offset: u64, config: C) -> Result<D> {
        let offset: usize = offset.try_into().map_err(|_| Error::OutOfRange)?;

        if self.mmap.len() > offset {
            Ok(decode_from_slice(&self.mmap[offset..], config)?.0)
        } else {
            Err(Error::OutOfRange)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use std::fs;
    use std::path::Path;

    const TEST_FILE_PATH: &str = "./test_output/store1.test";

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
    fn write_and_read() {
        for _ in 0..20 {
            clear_file();
            let path = Path::new(TEST_FILE_PATH);
            let data = (2..100000).fake::<String>();

            let mut file_store = FileStore::open(path).unwrap();
            file_store.write_bytes(data.as_bytes(), 0).unwrap();
            let d1 = file_store.read_bytes(1, data.len() as u64 / 2).unwrap();
            assert_eq!(&*d1, &data.as_bytes()[1..(data.len() / 2) + 1]);

            let mut mmap_store = MMapStore::open(path).unwrap();
            let d2 = mmap_store
                .read_bytes(data.len() as u64 / 2, (data.len() - data.len() / 2) as u64)
                .unwrap();
            assert_eq!(&*d2, &data.as_bytes()[data.len() / 2..]);
        }
    }
}
