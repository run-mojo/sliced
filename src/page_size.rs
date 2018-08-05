#![no_std]
//! This crate provides an easy, fast, cross-platform way to retrieve the
//! memory page size of the current system.
//!
//! Modern hardware and software tend to load data into RAM (and transfer data
//! from RAM to disk) in discrete chunk called pages. This crate provides a
//! helper method to retrieve the size in bytes of these pages. Since the page
//! size *should not* change during execution, this crate will cache the result
//! after it has been called once.
//!
//! To make this crate useful for writing memory allocators, it does not require
//! (but can use) the Rust standard library.
//!
//! Since Windows addresses sometimes have to correspond with an allocation
//! granularity that does not always match the size of the page, I have included
//! a method to retrieve that as well.
//!
//! # Example
//!
//! ```rust
//! extern crate page_size;
//! println!("{}", page_size::get());
//! ```

// `const_fn` is needed for `spin::Once`.
#![cfg_attr(feature = "no_std", feature(const_fn))]

#[cfg(feature = "no_std")]
use spin;
#[cfg(feature = "no_std")]
use spin::Once;

#[cfg(not(feature = "no_std"))]
use std;
#[cfg(not(feature = "no_std"))]
use std::sync::{Once, ONCE_INIT};

#[cfg(unix)]
use libc;

#[cfg(windows)]
use winapi;
#[cfg(target_os = "windows")]
use kernel32;

/// This function retrieves the system's memory page size.
///
/// # Example
///
/// ```rust
/// extern crate page_size;
/// println!("{}", page_size::get());
/// ```
pub fn get() -> usize {
    get_helper()
}

/// This function retrieves the system's memory allocation granularity.
///
/// # Example
///
/// ```rust
/// extern crate page_size;
/// println!("{}", page_size::get_granularity());
/// ```
pub fn get_granularity() -> usize {
    get_granularity_helper()
}

// Unix Section

#[cfg(all(unix, feature = "no_std"))]
#[inline]
fn get_helper() -> usize {
    static INIT: Once<usize> = Once::new();

    *INIT.call_once(unix::get)
}

#[cfg(all(unix, not(feature = "no_std")))]
#[inline]
fn get_helper() -> usize {
    static INIT: Once = ONCE_INIT;
    static mut PAGE_SIZE: usize = 0;

    unsafe {
        INIT.call_once(|| PAGE_SIZE = unix::get());
        PAGE_SIZE
    }
}

// Unix does not have a specific allocation granularity.
// The page size works well.
#[cfg(unix)]
#[inline]
fn get_granularity_helper() -> usize {
    get_helper()
}

#[cfg(unix)]
mod unix {
    use libc::{_SC_PAGESIZE, sysconf};

    #[inline]
    pub fn get() -> usize {
        unsafe {
            sysconf(_SC_PAGESIZE) as usize
        }
    }
}

// Windows Section

#[cfg(all(windows, feature = "no_std"))]
#[inline]
fn get_helper() -> usize {
    static INIT: Once<usize> = Once::new();

    *INIT.call_once(windows::get)
}

#[cfg(all(windows, not(feature = "no_std")))]
#[inline]
fn get_helper() -> usize {
    static INIT: Once = ONCE_INIT;
    static mut PAGE_SIZE: usize = 0;

    unsafe {
        INIT.call_once(|| PAGE_SIZE = windows::get());
        PAGE_SIZE
    }
}

#[cfg(all(windows, feature = "no_std"))]
#[inline]
fn get_granularity_helper() -> usize {
    static GRINIT: Once<usize> = Once::new();

    *GRINIT.call_once(windows::get_granularity)
}

#[cfg(all(windows, not(feature = "no_std")))]
#[inline]
fn get_granularity_helper() -> usize {
    static GRINIT: Once = ONCE_INIT;
    static mut GRANULARITY: usize = 0;

    unsafe {
        GRINIT.call_once(|| GRANULARITY = windows::get_granularity());
        GRANULARITY
    }
}

#[cfg(windows)]
mod windows {
    #[cfg(feature = "no_std")]
    use core::mem;
    #[cfg(not(feature = "no_std"))]
    use std::mem;

    use winapi::sysinfoapi::{SYSTEM_INFO, LPSYSTEM_INFO};
    use kernel32::GetSystemInfo;

    #[inline]
    pub fn get() -> usize {
        unsafe {
            let mut info: SYSTEM_INFO = mem::zeroed();
            GetSystemInfo(&mut info as LPSYSTEM_INFO);

            info.dwPageSize as usize
        }
    }

    #[inline]
    pub fn get_granularity() -> usize {
        unsafe {
            let mut info: SYSTEM_INFO = mem::zeroed();
            GetSystemInfo(&mut info as LPSYSTEM_INFO);

            info.dwAllocationGranularity as usize
        }
    }
}

// Stub Section

#[cfg(not(any(unix, windows)))]
#[inline]
fn get_helper() -> usize {
    4096 // 4k is the default on many systems
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        #[allow(unused_variables)]
        let page_size = get();
    }

    #[test]
    fn test_get_granularity() {
        #[allow(unused_variables)]
        let granularity = get_granularity();
    }
}
