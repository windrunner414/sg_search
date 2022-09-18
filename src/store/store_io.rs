use crate::store::{Error, Result};
use memmap2::{Mmap, MmapOptions};
use std::borrow::Cow;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub trait ReadableStore {
    fn read(&mut self, offset: u64, size: u64) -> Result<Cow<[u8]>>;
}

pub trait WriteableStore {
    fn write(&mut self, offset: u64, buf: &[u8]) -> Result<()>;

    fn append(&mut self, buf: &[u8]) -> Result<()>;
}

pub struct FileStore {
    file: File,
}

impl FileStore {
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
    fn read(&mut self, offset: u64, size: u64) -> Result<Cow<[u8]>> {
        let size: usize = size.try_into().map_err(|_| Error::OutOfRange)?;

        self.file.seek(SeekFrom::Start(offset))?;
        // notice: read to an uninitialized buffer is an undefined behavior
        let mut buf = vec![0u8; size];
        self.file.read_exact(&mut buf)?;

        Ok(Cow::Owned(buf))
    }
}

impl WriteableStore for FileStore {
    fn write(&mut self, offset: u64, buf: &[u8]) -> Result<()> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(buf)?;
        Ok(())
    }

    fn append(&mut self, buf: &[u8]) -> Result<()> {
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(buf)?;
        Ok(())
    }
}

pub struct MMapStore {
    mmap: Mmap,
}

impl MMapStore {
    pub fn open<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let file = File::open(file_path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(Self { mmap })
    }
}

impl ReadableStore for MMapStore {
    fn read(&mut self, offset: u64, size: u64) -> Result<Cow<[u8]>> {
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
            file_store.write(0, data.as_bytes()).unwrap();
            let d1 = file_store.read(1, data.len() as u64 / 2).unwrap();
            assert_eq!(&*d1, &data.as_bytes()[1..(data.len() / 2) + 1]);

            let mut mmap_store = MMapStore::open(path).unwrap();
            let d2 = mmap_store
                .read(data.len() as u64 / 2, (data.len() - data.len() / 2) as u64)
                .unwrap();
            assert_eq!(&*d2, &data.as_bytes()[data.len() / 2..]);
        }
    }
}
