use crate::store::Result;
use fs2::FileExt;
use std::fs::File;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct FileLock {
    file: File,
}

#[derive(Debug)]
pub struct FileLockReadGuard<'a> {
    file: &'a mut File,
}

#[derive(Debug)]
pub struct FileLockWriteGuard<'a> {
    file: &'a mut File,
}

impl FileLock {
    pub fn new(file: File) -> Self {
        Self { file }
    }

    pub fn read(&mut self) -> Result<FileLockReadGuard> {
        self.file.lock_shared()?;
        Ok(FileLockReadGuard {
            file: &mut self.file,
        })
    }

    pub fn try_read(&mut self) -> Result<FileLockReadGuard> {
        self.file.try_lock_shared()?;
        Ok(FileLockReadGuard {
            file: &mut self.file,
        })
    }

    pub fn write(&mut self) -> Result<FileLockWriteGuard> {
        self.file.lock_exclusive()?;
        Ok(FileLockWriteGuard {
            file: &mut self.file,
        })
    }

    pub fn try_write(&mut self) -> Result<FileLockWriteGuard> {
        self.file.try_lock_exclusive()?;
        Ok(FileLockWriteGuard {
            file: &mut self.file,
        })
    }
}

impl<'a> Deref for FileLockReadGuard<'a> {
    type Target = File;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.file
    }
}

impl<'a> DerefMut for FileLockReadGuard<'a> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.file
    }
}

impl<'a> Deref for FileLockWriteGuard<'a> {
    type Target = File;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.file
    }
}

impl<'a> DerefMut for FileLockWriteGuard<'a> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.file
    }
}

impl<'a> Drop for FileLockReadGuard<'a> {
    fn drop(&mut self) {
        self.file.unlock().expect("unlocking file error")
    }
}

impl<'a> Drop for FileLockWriteGuard<'a> {
    fn drop(&mut self) {
        self.file.unlock().expect("unlocking file error")
    }
}
