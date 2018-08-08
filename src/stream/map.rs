
use std::ptr;
use std::marker;
use std::mem;
use std::rc::Rc;

use libc;

use ::redis::rax::*;
use super::id::StreamID;

pub struct StreamIDMap<V> {
    pub rax: *mut rax,
    phantom: marker::PhantomData<V>,
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
extern "C" fn Sliced_StreamMap_FreeCallback<V>(v: *mut libc::c_void) {
    unsafe {
        // Decrement strong ref count
        Rc::from_raw(v as *const V);
    }
}

impl<V> Drop for StreamIDMap<V> {
    fn drop(&mut self) {
        unsafe {
            println!("dropped StreamMap(rax)");
            if !self.rax.is_null() {
                // Cleanup RAX
                raxFreeWithCallback(self.rax, Sliced_StreamMap_FreeCallback::<V>);
            }
        }
    }
}

#[inline]
fn to_be(id: &StreamID) -> StreamID {
    StreamID {
        ms: id.ms.to_be(),
        seq: id.seq.to_be(),
    }
}

/// Implementation of StreamMap
impl<V> StreamIDMap<V> {
    pub fn new() -> StreamIDMap<V> {
        unsafe {
            StreamIDMap {
                rax: raxNew(),
                phantom: marker::PhantomData,
            }
        }
    }

    /// The number of entries in the RAX
    pub fn len(&self) -> u64 {
        unsafe { raxSize(self.rax) }
    }

    /// The number of entries in the RAX
    pub fn size(&self) -> u64 {
        unsafe { raxSize(self.rax) }
    }

    /// Prints the Rax as ASCII art to stdout.
    pub fn show(&self) {
        unsafe { raxShow(self.rax) }
    }

    /// Insert or replace existing key with a NULL value.
    pub fn insert_null(&mut self, key: &StreamID) -> Result<Option<Rc<V>>, RaxError> {
        unsafe {
            // Allocate a pointer to catch the old value.
            let old: &mut *mut u8 = &mut ptr::null_mut();
            let mut k = key.to_big_endian();

            let r = raxInsert(
                self.rax,
                &mut k as *mut _ as *mut u8,
                mem::size_of::<StreamID>(),
                ptr::null_mut(),
                old,
            );

            if r == 0 && is_oom() {
                Err(RaxError::OutOfMemory())
            } else if old.is_null() {
                Ok(None)
            } else {
                // Box the previous value since Rax is done with it and it's our
                // responsibility now to drop it. Once this Box goes out of scope
                // the value is dropped and memory reclaimed.
                Ok(Some(Rc::from_raw(*old as *const V)))
            }
        }
    }

    /// Insert a new entry into the RAX if an existing one does not exist.
    pub fn try_insert(&mut self, key: &StreamID, data: Rc<V>) -> Result<Option<Rc<V>>, RaxError> {
        unsafe {
            // Allocate a pointer to catch the old value.
            let old: &mut *mut u8 = &mut ptr::null_mut();
            let value = Rc::into_raw(data);
            let mut k = key.to_big_endian();

            let r = raxTryInsert(
                self.rax,
                &mut k as *mut _ as *mut u8,
                mem::size_of::<StreamID>(),
                value as *mut V as *mut u8,
                old,
            );

            if r == 0 {
                if is_oom() {
                    Err(RaxError::OutOfMemory())
                } else {
                    Ok(Some(mem::transmute(value)))
                }
            } else if old.is_null() {
                Ok(None)
            } else {
                Ok(Some(Rc::from_raw(*old as *const V)))
            }
        }
    }

    /// Insert a new entry into the RAX replacing and returning the existing
    /// entry for the supplied key.
    pub fn insert(&mut self, key: &StreamID, data: Rc<V>) -> Result<Option<Rc<V>>, RaxError> {
        unsafe {
            // Allocate a pointer to catch the old value.
            let old: &mut *mut u8 = &mut ptr::null_mut();
            let value = Rc::into_raw(data);
            let mut k = key.to_big_endian();

            let r = raxInsert(
                self.rax,
                &mut k as *mut _ as *mut u8,
                mem::size_of::<StreamID>(),
                value as *mut V as *mut u8,
                old,
            );

            if r == 0 && is_oom() {
                Err(RaxError::OutOfMemory())
            } else if old.is_null() {
                Ok(None)
            } else {
                Ok(Some(Rc::from_raw(*old as *const V)))
            }
        }
    }

    ///
    ///
    ///
    pub fn remove(&mut self, key: &StreamID) -> (bool, Option<Rc<V>>) {
        unsafe {
            let old: &mut *mut u8 = &mut ptr::null_mut();
            let mut k = key.to_big_endian();

            let r = raxRemove(
                self.rax,
                &mut k as *mut _ as *mut u8,
                mem::size_of::<StreamID>(),
                old,
            );

            if old.is_null() {
                (r == 1, None)
            } else {
                (r == 1, Some(Rc::from_raw(*old as *const V)))
            }
        }
    }

    ///
    ///
    ///
    pub fn find_exists(&self, key: &StreamID) -> (bool, Option<Rc<V>>) {
        unsafe {
            let mut k = key.to_big_endian();

            let value = raxFind(
                self.rax,
                &mut k as *mut _ as *mut u8,
                mem::size_of::<StreamID>(),
            );

            if value.is_null() {
                (true, None)
            } else if value == raxNotFound {
                (false, None)
            } else {
                // transmute to the value so we don't drop the actual value accidentally.
                // While the key associated to the value is in the RAX then we cannot
                // drop it.
                (true, Some(Rc::from_raw(value as *const _ as *const V).clone()))
            }
        }
    }

    /// Same as get but added for semantics parity.
    pub fn find(&self, key: &StreamID) -> Option<Rc<V>> {
        unsafe {
            let mut k = key.to_big_endian();

            let value = raxFind(
                self.rax,
                &mut k as *mut _ as *mut u8,
                mem::size_of::<StreamID>(),
            );

            if value.is_null() || value == raxNotFound {
                None
            } else {
                // transmute to the value so we don't drop the actual value accidentally.
                // While the key associated to the value is in the RAX then we cannot
                // drop it.
//                Some(std::mem::transmute(value))
                Some(Rc::from_raw(value as *const _ as *const V).clone())
            }
        }
    }

    ///
    ///
    ///
    pub fn get(&self, key: &StreamID) -> Option<Rc<V>> {
        unsafe {
            let mut k = key.to_big_endian();

            let value = raxFind(
                self.rax,
                &mut k as *mut _ as *mut u8,
                mem::size_of::<StreamID>(),
            );

            if value.is_null() || value == raxNotFound {
                None
            } else {
                // transmute to the value so we don't drop the actual value accidentally.
                // While the key associated to the value is in the RAX then we cannot
                // drop it.
                Some(Rc::from_raw(value as *const _ as *const V).clone())
            }
        }
    }

    /// Determines if the supplied key exists in the Rax.
    pub fn exists(&self, key: &StreamID) -> bool {
        unsafe {
            let mut k = key.to_big_endian();

            let value = raxFind(
                self.rax,
                &mut k as *mut _ as *mut u8,
                mem::size_of::<StreamID>(),
            );

            if value.is_null() || value == raxNotFound {
                false
            } else {
                true
            }
        }
    }

    ///
    #[inline]
    pub fn first_key(
        &self
    ) -> Option<StreamID> {
        unsafe {
            // Allocate stack memory.
            let mut iter: RaxIterator<StreamID, V> = mem::uninitialized();
            // Initialize a Rax iterator. This call should be performed a single time
            // to initialize the iterator, and must be followed by a raxSeek() call,
            // otherwise the raxPrev()/raxNext() functions will just return EOF.
            raxStart(&iter as *const _ as *const raxIterator, self.rax);
            if raxSeek(
                &iter as *const _ as *const raxIterator,
                BEGIN.as_ptr(),
                ptr::null_mut(),
                0
            ) != 0 || iter.key_len == 0 {
                None
            } else {
                Some(StreamID::from_buf(iter.key, iter.key_len))
            }
        }
    }

    ///
    #[inline]
    pub fn last_key(
        &self
    ) -> Option<StreamID> {
        unsafe {
            // Allocate stack memory.
            let mut iter: RaxIterator<StreamID, V> = mem::uninitialized();
            // Initialize a Rax iterator. This call should be performed a single time
            // to initialize the iterator, and must be followed by a raxSeek() call,
            // otherwise the raxPrev()/raxNext() functions will just return EOF.
            raxStart(&iter as *const _ as *const raxIterator, self.rax);
            if raxSeek(&iter as *const _ as *const raxIterator, END.as_ptr(), ptr::null_mut(), 0) != 0 || iter.key_len == 0 {
                None
            } else {
                Some(StreamID::from_buf(iter.key, iter.key_len))
            }
        }
    }

    ///
    #[inline]
    pub fn seek<F>(
        &self,
        op: &str,
        key: StreamID,
        f: F,
    ) where
        F: Fn(
            &StreamIDMap<V>,
            &mut RaxIterator<StreamID, V>,
        ) {
        unsafe {
            // Allocate stack memory.
            let mut iter: RaxIterator<StreamID, V> = mem::uninitialized();
            // Initialize a Rax iterator. This call should be performed a single time
            // to initialize the iterator, and must be followed by a raxSeek() call,
            // otherwise the raxPrev()/raxNext() functions will just return EOF.
            raxStart(&iter as *const _ as *const raxIterator, self.rax);
            iter.seek(op, key);
            // Borrow stack iterator and execute the closure.
            f(self, &mut iter)
        }
    }

    ///
    #[inline]
    pub fn seek_mut<F>(
        &mut self,
        op: &str,
        key: StreamID,
        f: F,
    ) where
        F: Fn(
            &mut StreamIDMap<V>,
            &mut RaxIterator<StreamID, V>,
        ) {
        unsafe {
            // Allocate stack memory.
            let mut iter: RaxIterator<StreamID, V> = mem::uninitialized();
            // Initialize a Rax iterator. This call should be performed a single time
            // to initialize the iterator, and must be followed by a raxSeek() call,
            // otherwise the raxPrev()/raxNext() functions will just return EOF.
            raxStart(&iter as *const _ as *const raxIterator, self.rax);
            iter.seek(op, key);
            // Borrow stack iterator and execute the closure.
            f(self, &mut iter)
        }
    }

    ///
    #[inline]
    pub fn seek_result<R, F>(
        &self,
        op: &str,
        key: StreamID,
        f: F,
    ) -> Result<R, RaxError>
        where
            F: Fn(
                &StreamIDMap<V>,
                &mut RaxIterator<StreamID, V>,
            ) -> Result<R, RaxError> {
        unsafe {
            // Allocate stack memory.
            let mut iter: RaxIterator<StreamID, V> = mem::uninitialized();
            // Initialize a Rax iterator. This call should be performed a single time
            // to initialize the iterator, and must be followed by a raxSeek() call,
            // otherwise the raxPrev()/raxNext() functions will just return EOF.
            raxStart(&iter as *const _ as *const raxIterator, self.rax);
            iter.seek(op, key);
            // Borrow stack iterator and execute the closure.
            f(self, &mut iter)
        }
    }

    ///
    #[inline]
    pub fn seek_result_mut<R, F>(
        &mut self,
        op: &str,
        key: StreamID,
        f: F,
    ) -> Result<R, RaxError>
        where
            F: Fn(
                &mut StreamIDMap<V>,
                &mut RaxIterator<StreamID, V>,
            ) -> Result<R, RaxError> {
        unsafe {
            // Allocate stack memory.
            let mut iter: RaxIterator<StreamID, V> = mem::uninitialized();
            // Initialize a Rax iterator. This call should be performed a single time
            // to initialize the iterator, and must be followed by a raxSeek() call,
            // otherwise the raxPrev()/raxNext() functions will just return EOF.
            raxStart(&iter as *const _ as *const raxIterator, self.rax);
            iter.seek(op, key);
            // Borrow stack iterator and execute the closure.
            f(self, &mut iter)
        }
    }
}