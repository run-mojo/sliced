use std;
use std::alloc;
//use super::boxed::*;
use std::mem;
use std::ptr;
use std::ptr::{NonNull};
use std::alloc::{Layout, AllocErr};


/// listpacks are contiguous chunks of memory. The "Allocator" controls the
/// behavior and system for allocating, re-allocating, and de-allocating
/// listpacks. All the write methods within the "raw" module deal with raw
/// pointers on listpack allocations. It is agnostic to where that allocation
/// came from and can be used for mmap files as well.
pub trait Allocator: Sized {
    fn alloc(&self, size: usize) -> *mut u8;

    fn realloc(&self, ptr: *mut u8, size: usize) -> *mut u8;

    fn dealloc(&self, lp: *mut u8);
}

/// Default listpack allocator that uses the system allocator.
pub struct Rallocator;

///
pub static mut ALLOCATOR: &'static Rallocator = &Rallocator;

#[inline(always)]
pub fn alloc(size: usize) -> *mut u8 {
    unsafe { ::redis::api::redis_malloc(size) }
}

#[inline(always)]
pub fn non_zeroed<'a, T>() -> &'a mut T where T: Sized {
    unsafe { &mut *(alloc(std::mem::size_of::<T>()) as *mut T) }
}

#[inline(always)]
pub fn zeroed<'a, T>() -> &'a mut T where T: Sized {
    unsafe {
        let p = &mut *(alloc(std::mem::size_of::<T>()) as *mut T);
        std::ptr::write_bytes(p, 0, std::mem::size_of::<T>());
        p
    }
}

/// Use the Redis Memory allocator to allocate heap memory
/// and copy the value passed in to it.
#[inline(always)]
pub fn leak<'a, T>(val: T) -> &'a mut T
    where T: Sized {
    unsafe {
        // Create a heap allocation.
        let p = non_zeroed();
        // Copy in place.
        ptr::copy_nonoverlapping(
            &val as *const _ as *const u8,
            p as *mut _ as *mut u8,
            std::mem::size_of::<T>(),
        );
        // Protect against double free.
        mem::forget(val);
        p
    }
}

#[inline(always)]
pub fn leak_raw<T>(val: T) -> *mut T
    where T: Sized {
    unsafe {
        // Create a heap allocation.
        let p = non_zeroed();
        // Copy in place.
        ptr::copy_nonoverlapping(
            &val as *const _ as *const u8,
            p as *mut _ as *mut u8,
            std::mem::size_of::<T>(),
        );
        // Protect against double free.
        mem::forget(val);
        p as *mut _ as *mut T
    }
}

///
#[inline(always)]
pub fn boxed<'a, T>(val: T) -> Box<T>
    where T: Sized {
    unsafe {
        // Create a heap allocation.
        let p = alloc(std::mem::size_of::<T>());

        // Copy in place.
        ptr::copy_nonoverlapping(
            &val as *const _ as *const u8,
            p,
            std::mem::size_of::<T>(),
        );

        // Protect against double free.
        mem::forget(val);
        Box::from_raw(p as *mut T)
    }
}

///
#[inline(always)]
pub fn boxed_non_null<'a, T>(val: T) -> std::ptr::NonNull<T>
    where T: Sized {
    unsafe {
        // Create a heap allocation.
        let p = alloc(std::mem::size_of::<T>());

        // Copy in place.
        ptr::copy_nonoverlapping(
            &val as *const _ as *const u8,
            p,
            std::mem::size_of::<T>(),
        );

        // Protect against double free.
        mem::forget(val);
        Box::into_raw_non_null(
            Box::from_raw(p as *mut T)
        )
    }
}

#[inline(always)]
pub fn free<'a, T>(val: &'a mut T) where T: Sized {
    unsafe {
        dealloc(val as *mut _ as *mut u8)
    }
}

#[inline]
pub(crate) unsafe fn box_free<T: ?Sized>(p: std::ptr::Unique<T>) {
    dealloc(p.as_ptr() as *mut _ as *mut u8);
}


#[inline(always)]
pub fn realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    unsafe { ::redis::api::redis_realloc(ptr, size) }
}

#[inline(always)]
pub fn dealloc(ptr: *mut u8) {
    unsafe { ::redis::api::redis_free(ptr) }
}

impl Allocator for Rallocator {
    #[inline(always)]
    fn alloc(&self, size: usize) -> *mut u8 {
        unsafe { ::redis::api::redis_malloc(size) }
    }

    #[inline(always)]
    fn realloc(&self, lp: *mut u8, size: usize) -> *mut u8 {
        unsafe { ::redis::api::redis_realloc(lp, size) }
    }

    #[inline]
    fn dealloc(&self, lp: *mut u8) {
        unsafe { ::redis::api::redis_free(lp) }
    }
}


pub struct RustAllocator;
pub const RUST_ALLOCATOR: RustAllocator = RustAllocator;

unsafe impl std::alloc::Alloc for RustAllocator {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        NonNull::new(alloc(layout.size())).ok_or(AllocErr)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        dealloc(ptr.as_ptr())
    }

    #[inline]
    unsafe fn realloc(&mut self, ptr: NonNull<u8>, layout: Layout, new_size: usize) -> Result<NonNull<u8>, AllocErr> {
        NonNull::new(realloc(ptr.as_ptr(), new_size)).ok_or(AllocErr)
    }
}