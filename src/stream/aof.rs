use std::fs::{File};
use std::path::Path;
use std::io;
use std::io::Error as IoError;
use std::io::Result as IoResult;
use std::ptr;
use ::mmap::{Mmap, MmapMut, MmapOptions};
use ::redis::listpack;
use spin::Mutex;

const AOF_GROW_1MB: u64 = 1024 * 1024;
const AOF_GROW_2MB: u64 = 1024 * 1024 * 2;
const AOF_GROW_4MB: u64 = 1024 * 1024 * 4;
const AOF_GROW_8MB: u64 = 1024 * 1024 * 8;
const AOF_GROW_16MB: u64 = 1024 * 1024 * 16;
const AOF_GROW_32MB: u64 = 1024 * 1024 * 32;
const AOF_GROW_64MB: u64 = 1024 * 1024 * 64;

/// Memory mapped Append-only file.
pub struct AOF {
    inner: Mutex<AOFInner>
}

impl AOF {
    pub fn new(f: File, size: u64) -> IoResult<AOF> {
        let len = f.metadata().unwrap().len();

        // Truncate
        match len {
            0 => {
                f.set_len(size);
            }
            _ => {
                // Let's not allow shrinking here.
                if len > size {
                    return Err(IoError::from(io::ErrorKind::UnexpectedEof))
                }
                match f.set_len(size) {
                    Err(e) => return Err(e),
                    _ => {}
                }
            }
        }

        match unsafe { MmapOptions::new().map_mut(&f) } {
            Ok(map) => Ok(AOF {
                inner: Mutex::new(AOFInner {
                    file: f,
                    mmap: map,
                    offset: len,
                    length: len,
                    size: len,
                })
            }),
            Err(e) => Err(e)
        }
    }

    pub fn try_read(&self, offset: u64, buf: *mut u8, size: usize) -> IoResult<()> {

        Ok(())
    }

    ///
    pub fn try_append(&self, buf: *mut u8, size: usize) -> IoResult<bool> {
        let mut lock = self.inner.lock();

        let mmap = lock.mmap.as_mut_ptr();
        if lock.length + (size as u64) > lock.size + 1 {
            drop(lock);
            // Need to grow the file or use the next one.
            return Ok(false)
        }

        unsafe {
            // memcpy
            ptr::copy_nonoverlapping(
                buf as *const u8,
                mmap.offset(lock.length as isize),
                size
            );
            // Move the EOF byte to the new end.
            *mmap.offset(lock.length as isize + 1) = listpack::EOF;
        }

        // Do not include the EOF byte so it will be overwritten.
        lock.length = lock.length + (size as u64);

        drop(lock);
        Ok(true)
    }

    ///
    pub fn truncate(&self, size: u64) -> IoResult<()> {
        let mut lock = self.inner.lock();

        // Truncate the file.
        (&mut lock.file).set_len(size);

        // fsync existing contents.
        lock.mmap.flush();

        // mmap the whole file.
        match unsafe { MmapOptions::new()
            .offset(0)
            .len(size as usize)
            .map_mut(&lock.file)
        } {
            Ok(map) => {
                lock.mmap = map;
                drop(lock);
                return Ok(())
            }
            Err(e) => {
                drop(lock);
                return Err(e)
            }
        }
    }
}

struct AOFInner {
    file: File,
    mmap: MmapMut,
    /// The logical tail of the file is mlock'ed into memory to allow
    /// writes directly from an event-loop.
    offset: u64,
    length: u64,
    size: u64,
}