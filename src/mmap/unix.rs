use libc;
use std::{io, ptr};
use std::fs::File;
use std::os::unix::io::{AsRawFd, RawFd};

#[cfg(any(all(target_os = "linux", not(target_arch = "mips")), target_os = "freebsd",
target_os = "android"))]
const MAP_STACK: libc::c_int = libc::MAP_STACK;

#[cfg(not(any(all(target_os = "linux", not(target_arch = "mips")), target_os = "freebsd",
target_os = "android")))]
const MAP_STACK: libc::c_int = 0;

pub struct MmapInner {
    ptr: *mut libc::c_void,
    len: usize,
}

impl MmapInner {
    /// Creates a new `MmapInner`.
    ///
    /// This is a thin wrapper around the `mmap` sytem call.
    fn new(
        len: usize,
        prot: libc::c_int,
        flags: libc::c_int,
        file: RawFd,
        offset: usize,
    ) -> io::Result<MmapInner> {
        let alignment = offset % page_size();
        let aligned_offset = offset - alignment;
        let aligned_len = len + alignment;
        if aligned_len == 0 {
            // Normally the OS would catch this, but it segfaults under QEMU.
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "memory map must have a non-zero length",
            ));
        }

        unsafe {
            let ptr = libc::mmap(
                ptr::null_mut(),
                aligned_len as libc::size_t,
                prot,
                flags,
                file,
                aligned_offset as libc::off_t,
            );

            if ptr == libc::MAP_FAILED {
                Err(io::Error::last_os_error())
            } else {
                Ok(MmapInner {
                    ptr: ptr.offset(alignment as isize),
                    len: len,
                })
            }
        }
    }

    pub fn map(len: usize, file: &File, offset: usize) -> io::Result<MmapInner> {
        MmapInner::new(
            len,
            libc::PROT_READ,
            libc::MAP_SHARED,
            file.as_raw_fd(),
            offset,
        )
    }

    pub fn map_exec(len: usize, file: &File, offset: usize) -> io::Result<MmapInner> {
        MmapInner::new(
            len,
            libc::PROT_READ | libc::PROT_EXEC,
            libc::MAP_SHARED,
            file.as_raw_fd(),
            offset,
        )
    }

    pub fn map_mut(len: usize, file: &File, offset: usize) -> io::Result<MmapInner> {
        MmapInner::new(
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            file.as_raw_fd(),
            offset,
        )
    }

    pub fn map_copy(len: usize, file: &File, offset: usize) -> io::Result<MmapInner> {
        MmapInner::new(
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE,
            file.as_raw_fd(),
            offset,
        )
    }

    /// Open an anonymous memory map.
    pub fn map_anon(len: usize, stack: bool) -> io::Result<MmapInner> {
        let stack = if stack { MAP_STACK } else { 0 };
        MmapInner::new(
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED | libc::MAP_ANON | stack,
            -1,
            0,
        )
    }

    pub fn flush(&self, offset: usize, len: usize) -> io::Result<()> {
        let alignment = (self.ptr as usize + offset) % page_size();
        let offset = offset as isize - alignment as isize;
        let len = len + alignment;
        let result =
            unsafe { libc::msync(self.ptr.offset(offset), len as libc::size_t, libc::MS_SYNC) };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn flush_async(&self, offset: usize, len: usize) -> io::Result<()> {
        let alignment = offset % page_size();
        let aligned_offset = offset - alignment;
        let aligned_len = len + alignment;
        let result = unsafe {
            libc::msync(
                self.ptr.offset(aligned_offset as isize),
                aligned_len as libc::size_t,
                libc::MS_ASYNC,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn is_resident(&self, offset: usize, len: usize) -> bool {
        let vec_size = (len + page_size() - 1) / page_size();

        unsafe {
            if vec_size < 17 {
                let v = &mut [0i8; 16];
                if libc::mincore(self.ptr.offset(offset as isize), len, v.as_mut_ptr()) != 0 {
                    return false;
                }
                for i in 0..vec_size {
                    if v[i] == 0 {
                        return false
                    }
                }
                true
            } else if vec_size < 65 {
                let v = &mut [0i8; 64];
                if libc::mincore(self.ptr.offset(offset as isize), len, v.as_mut_ptr()) != 0 {
                    return false;
                }
                for i in 0..vec_size {
                    if v[i] == 0 {
                        return false
                    }
                }
                true
            } else if vec_size < 257 {
                let v = &mut [0i8; 256];
                if libc::mincore(self.ptr.offset(offset as isize), len, v.as_mut_ptr()) != 0 {
                    return false;
                }
                for i in 0..vec_size {
                    if v[i] == 0 {
                        return false
                    }
                }
                true
            } else if vec_size < 1025 {
                let v = &mut [0i8; 1024];
                if libc::mincore(self.ptr.offset(offset as isize), len, v.as_mut_ptr()) != 0 {
                    return false;
                }
                for i in 0..vec_size {
                    if v[i] == 0 {
                        return false
                    }
                }
                true
            } else if vec_size < 4097 {
                let v = &mut [0i8; 4096];
                if libc::mincore(self.ptr.offset(offset as isize), len, v.as_mut_ptr()) != 0 {
                    return false;
                }
                for i in 0..vec_size {
                    if v[i] == 0 {
                        return false
                    }
                }
                true
            } else {
                false
            }
        }
    }

    fn mprotect(&mut self, prot: libc::c_int) -> io::Result<()> {
        unsafe {
            let alignment = self.ptr as usize % page_size();
            let ptr = self.ptr.offset(-(alignment as isize));
            let len = self.len + alignment;
            let mincore = libc::mincore;
            if libc::mprotect(ptr, len, prot) == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    pub fn make_read_only(&mut self) -> io::Result<()> {
        self.mprotect(libc::PROT_READ)
    }

    pub fn make_exec(&mut self) -> io::Result<()> {
        self.mprotect(libc::PROT_READ | libc::PROT_EXEC)
    }

    pub fn make_mut(&mut self) -> io::Result<()> {
        self.mprotect(libc::PROT_READ | libc::PROT_WRITE)
    }

    #[inline]
    pub fn ptr(&self) -> *const u8 {
        self.ptr as *const u8
    }

    #[inline]
    pub fn mut_ptr(&mut self) -> *mut u8 {
        self.ptr as *mut u8
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl Drop for MmapInner {
    fn drop(&mut self) {
        let alignment = self.ptr as usize % page_size();
        unsafe {
            println!("MmapInner Drop!");
            assert!(
                libc::munmap(
                    self.ptr.offset(-(alignment as isize)),
                    (self.len + alignment) as libc::size_t,
                ) == 0,
                "unable to unmap mmap: {}",
                io::Error::last_os_error()
            );
        }
    }
}

unsafe impl Sync for MmapInner {}

unsafe impl Send for MmapInner {}

fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}
