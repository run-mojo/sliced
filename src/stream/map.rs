use crate::redis::rax::*;
use libc;
use std::marker;
use std::mem;
use std::ptr;
use std::rc::Rc;
use super::*;
use super::id::StreamID;

pub struct StreamIDRax<V> {
    pub rax: *mut rax,
    _phantom: marker::PhantomData<V>,
}

pub trait RaxKey<RHS = Self>: Clone + Default + Send + Sync {
    fn into_rax(&mut self) -> (*const u8, usize);

    fn from_rax(ptr: *const u8, len: usize) -> RHS;
}


impl RaxKey for u64 {
    #[inline]
    fn into_rax(&mut self) -> (*const u8, usize) {
        *self = self.to_be();
        (self as *const _ as *const u8, mem::size_of::<Self>())
    }

    #[inline]
    fn from_rax(ptr: *const u8, len: usize) -> Self {
        if len != mem::size_of::<Self>() {
            return Self::default();
        }
        unsafe { Self::from_be(*(ptr as *mut [u8; mem::size_of::<Self>()] as *mut Self)) }
    }
}

impl RaxKey for StreamID {
    #[inline]
    fn into_rax(&mut self) -> (*const u8, usize) {
        *self = StreamID {
            ms: self.ms.to_be(),
            seq: self.seq.to_be(),
        };

        (self as *const _ as *const u8, mem::size_of::<StreamID>())
    }

    #[inline]
    fn from_rax(ptr: *const u8, len: usize) -> StreamID {
        if len != mem::size_of::<StreamID>() {
            return StreamID::default();
        }

        unsafe {
            StreamID {
                ms: u64::from_be(*(ptr as *mut [u8; 8] as *mut u64)),
                seq: u64::from_be(*(ptr.offset(8) as *mut [u8; 8] as *mut u64)),
            }
        }
    }
}

impl RaxKey for SDS {
    #[inline]
    fn into_rax(&mut self) -> (*const u8, usize) {
        (self.as_ptr(), self.len())
    }

    #[inline]
    fn from_rax(ptr: *const u8, len: usize) -> SDS {
        SDS::from_ptr(ptr, len)
    }
}

impl<'a> RaxKey for &'a str {
    #[inline]
    fn into_rax(&mut self) -> (*const u8, usize) {
        (self.as_ptr(), self.len())
    }

    #[inline]
    fn from_rax(ptr: *const u8, len: usize) -> &'a str {
        unsafe {
            std::str::from_utf8_unchecked(
                std::slice::from_raw_parts(ptr, len)
            )
        }
    }
}

impl<'a> RaxKey for &'a [u8] {
    #[inline]
    fn into_rax(&mut self) -> (*const u8, usize) {
        (self.as_ptr(), self.len())
    }

    #[inline]
    fn from_rax(ptr: *const u8, len: usize) -> &'a [u8] {
        unsafe {
            std::slice::from_raw_parts(ptr, len)
        }
    }
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
extern "C" fn Sliced_StreamIDRax_FreeCallback<V>(v: *mut libc::c_void) {
    unsafe {
        // Decrement strong ref count
        Rc::from_raw(v as *const V);
    }
}

impl<V> Drop for StreamIDRax<V> {
    fn drop(&mut self) {
        unsafe {
            println!("dropped StreamMap(rax)");
            if !self.rax.is_null() {
                // Cleanup RAX
                raxFreeWithCallback(self.rax, Sliced_StreamIDRax_FreeCallback::<V>);
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
impl<V> StreamIDRax<V> {
    pub fn new() -> StreamIDRax<V> {
        unsafe {
            StreamIDRax {
                rax: raxNew(),
                _phantom: marker::PhantomData,
            }
        }
    }

//    /// The number of entries in the RAX
//    pub fn len(&self) -> u64 {
//        unsafe { raxSize(self.rax) }
//    }
//
//    /// The number of entries in the RAX
//    pub fn size(&self) -> u64 {
//        unsafe { raxSize(self.rax) }
//    }
//
//    /// Prints the Rax as ASCII art to stdout.
//    pub fn show(&self) {
//        unsafe { raxShow(self.rax) }
//    }
//
//    /// Insert or replace existing key with a NULL value.
//    pub fn insert_null(&mut self, key: &StreamID) -> Result<Option<Rc<V>>, StreamError> {
//        unsafe {
//            // Allocate a pointer to catch the old value.
//            let old: &mut *mut u8 = &mut ptr::null_mut();
//            let mut k = key.to_big_endian();
//
//            let r = raxInsert(
//                self.rax,
//                &mut k as *mut _ as *mut u8,
//                mem::size_of::<StreamID>(),
//                ptr::null_mut(),
//                old,
//            );
//
//            if r == 0 && is_oom() {
//                Err(StreamError::OutOfMemory)
//            } else if old.is_null() {
//                Ok(None)
//            } else {
//                // Box the previous value since Rax is done with it and it's our
//                // responsibility now to drop it. Once this Box goes out of scope
//                // the value is dropped and memory reclaimed.
//                Ok(Some(Rc::from_raw(*old as *const V)))
//            }
//        }
//    }
//
//    /// Insert a new entry into the RAX if an existing one does not exist.
//    pub fn try_insert(&mut self, key: &StreamID, data: Rc<V>) -> Result<Option<Rc<V>>, StreamError> {
//        unsafe {
//            // Allocate a pointer to catch the old value.
//            let old: &mut *mut u8 = &mut ptr::null_mut();
//            let value = Rc::into_raw(data);
//            let mut k = key.to_big_endian();
//
//            let r = raxTryInsert(
//                self.rax,
//                &mut k as *mut _ as *mut u8,
//                mem::size_of::<StreamID>(),
//                value as *mut V as *mut u8,
//                old,
//            );
//
//            if r == 0 {
//                if is_oom() {
//                    Err(StreamError::OutOfMemory)
//                } else {
//                    Ok(Some(mem::transmute(value)))
//                }
//            } else if old.is_null() {
//                Ok(None)
//            } else {
//                Ok(Some(Rc::from_raw(*old as *const V)))
//            }
//        }
//    }
//
//    /// Insert a new entry into the RAX replacing and returning the existing
//    /// entry for the supplied key.
//    pub fn insert(&mut self, key: &StreamID, data: Rc<V>) -> Result<Option<Rc<V>>, RaxError> {
//        unsafe {
//            // Allocate a pointer to catch the old value.
//            let old: &mut *mut u8 = &mut ptr::null_mut();
//            let value = Rc::into_raw(data);
//            let mut k = key.to_big_endian();
//
//            let r = raxInsert(
//                self.rax,
//                &mut k as *mut _ as *mut u8,
//                mem::size_of::<StreamID>(),
//                value as *mut V as *mut u8,
//                old,
//            );
//
//            if r == 0 && is_oom() {
//                Err(RaxError::OutOfMemory)
//            } else if old.is_null() {
//                Ok(None)
//            } else {
//                Ok(Some(Rc::from_raw(*old as *const V)))
//            }
//        }
//    }
//
//    ///
//    ///
//    ///
//    pub fn remove(&mut self, key: &StreamID) -> (bool, Option<Rc<V>>) {
//        unsafe {
//            let old: &mut *mut u8 = &mut ptr::null_mut();
//            let mut k = key.to_big_endian();
//
//            let r = raxRemove(
//                self.rax,
//                &mut k as *mut _ as *mut u8,
//                mem::size_of::<StreamID>(),
//                old,
//            );
//
//            if old.is_null() {
//                (r == 1, None)
//            } else {
//                (r == 1, Some(Rc::from_raw(*old as *const V)))
//            }
//        }
//    }
//
//    ///
//    ///
//    ///
//    pub fn find_exists(&self, key: &StreamID) -> (bool, Option<Rc<V>>) {
//        unsafe {
//            let mut k = key.to_big_endian();
//
//            let value = raxFind(
//                self.rax,
//                &mut k as *mut _ as *mut u8,
//                mem::size_of::<StreamID>(),
//            );
//
//            if value.is_null() {
//                (true, None)
//            } else if value == raxNotFound {
//                (false, None)
//            } else {
//                // transmute to the value so we don't drop the actual value accidentally.
//                // While the key associated to the value is in the RAX then we cannot
//                // drop it.
//                (true, Some(Rc::from_raw(value as *const _ as *const V).clone()))
//            }
//        }
//    }
//
//    /// Same as get but added for semantics parity.
//    pub fn find(&self, key: &StreamID) -> Option<Rc<V>> {
//        unsafe {
//            let mut k = key.to_big_endian();
//
//            let value = raxFind(
//                self.rax,
//                &mut k as *mut _ as *mut u8,
//                mem::size_of::<StreamID>(),
//            );
//
//            if value.is_null() || value == raxNotFound {
//                None
//            } else {
//                // transmute to the value so we don't drop the actual value accidentally.
//                // While the key associated to the value is in the RAX then we cannot
//                // drop it.
////                Some(std::mem::transmute(value))
//                Some(Rc::from_raw(value as *const _ as *const V).clone())
//            }
//        }
//    }
//
//    ///
//    ///
//    ///
//    pub fn get(&self, key: &StreamID) -> Option<Rc<V>> {
//        unsafe {
//            let mut k = key.to_big_endian();
//
//            let value = raxFind(
//                self.rax,
//                &mut k as *mut _ as *mut u8,
//                mem::size_of::<StreamID>(),
//            );
//
//            if value.is_null() || value == raxNotFound {
//                None
//            } else {
//                // transmute to the value so we don't drop the actual value accidentally.
//                // While the key associated to the value is in the RAX then we cannot
//                // drop it.
//                Some(Rc::from_raw(value as *const _ as *const V).clone())
//            }
//        }
//    }
//
//    /// Determines if the supplied key exists in the Rax.
//    pub fn exists(&self, key: &StreamID) -> bool {
//        unsafe {
//            let mut k = key.to_big_endian();
//
//            let value = raxFind(
//                self.rax,
//                &mut k as *mut _ as *mut u8,
//                mem::size_of::<StreamID>(),
//            );
//
//            if value.is_null() || value == raxNotFound {
//                false
//            } else {
//                true
//            }
//        }
//    }

//    ///
//    #[inline]
//    pub fn first_key(
//        &self
//    ) -> Option<StreamID> {
//        unsafe {
//            // Allocate stack memory.
//            let mut iter: RaxIter<StreamID, V> = mem::uninitialized();
//            // Initialize a Rax iterator. This call should be performed a single time
//            // to initialize the iterator, and must be followed by a raxSeek() call,
//            // otherwise the raxPrev()/raxNext() functions will just return EOF.
//            raxStart(&iter as *const _ as *const raxIterator, self.rax);
//            if raxSeek(
//                &iter as *const _ as *const raxIterator,
//                BEGIN.as_ptr(),
//                ptr::null_mut(),
//                0
//            ) != 0 || iter.key_len == 0 {
//                None
//            } else {
//                Some(StreamID::from_rax(iter.key, iter.key_len))
//            }
//        }
//    }
//
//    ///
//    #[inline]
//    pub fn last_key(
//        &self
//    ) -> Option<StreamID> {
//        unsafe {
//            // Allocate stack memory.
//            let mut iter: RaxIter<StreamID, V> = mem::uninitialized();
//            // Initialize a Rax iterator. This call should be performed a single time
//            // to initialize the iterator, and must be followed by a raxSeek() call,
//            // otherwise the raxPrev()/raxNext() functions will just return EOF.
//            raxStart(&iter as *const _ as *const raxIterator, self.rax);
//            if raxSeek(&iter as *const _ as *const raxIterator, END.as_ptr(), ptr::null_mut(), 0) != 0 || iter.key_len == 0 {
//                None
//            } else {
//                Some(StreamID::from_raw(iter.key, iter.key_len))
//            }
//        }
//    }
//
//    ///
//    #[inline]
//    pub fn seek<F>(
//        &self,
//        op: &str,
//        key: StreamID,
//        f: F,
//    ) where
//        F: Fn(
//            &StreamIDRax<V>,
//            &mut RaxIterator<StreamID, V>,
//        ) {
//        unsafe {
//            // Allocate stack memory.
//            let mut iter: RaxIter<StreamID, V> = mem::uninitialized();
//            // Initialize a Rax iterator. This call should be performed a single time
//            // to initialize the iterator, and must be followed by a raxSeek() call,
//            // otherwise the raxPrev()/raxNext() functions will just return EOF.
//            raxStart(&iter as *const _ as *const raxIterator, self.rax);
//            iter.seek(op, key);
//            // Borrow stack iterator and execute the closure.
//            f(self, &mut iter)
//        }
//    }
//
//    ///
//    #[inline]
//    pub fn seek_mut<F>(
//        &mut self,
//        op: &str,
//        key: &mut StreamID,
//        f: F,
//    ) where
//        F: Fn(
//            &mut StreamIDRax<V>,
//            &mut RaxIterator<StreamID, V>,
//        ) {
//        unsafe {
//            // Allocate stack memory.
//            let mut iter: RaxIterator<StreamID, V> = mem::uninitialized();
//            // Initialize a Rax iterator. This call should be performed a single time
//            // to initialize the iterator, and must be followed by a raxSeek() call,
//            // otherwise the raxPrev()/raxNext() functions will just return EOF.
//            raxStart(&iter as *const _ as *const raxIterator, self.rax);
//            iter.seek(op, key);
//            // Borrow stack iterator and execute the closure.
//            f(self, &mut iter)
//        }
//    }
//
//    ///
//    #[inline]
//    pub fn seek_result<R, F>(
//        &self,
//        op: &str,
//        key: StreamID,
//        f: F,
//    ) -> Result<R, RaxError>
//        where
//            F: Fn(
//                &StreamIDRax<V>,
//                &mut RaxIterator<StreamID, V>,
//            ) -> Result<R, RaxError> {
//        unsafe {
//            // Allocate stack memory.
//            let mut iter: RaxIterator<StreamID, V> = mem::uninitialized();
//            // Initialize a Rax iterator. This call should be performed a single time
//            // to initialize the iterator, and must be followed by a raxSeek() call,
//            // otherwise the raxPrev()/raxNext() functions will just return EOF.
//            raxStart(&iter as *const _ as *const raxIterator, self.rax);
//            iter.seek(op, key);
//            // Borrow stack iterator and execute the closure.
//            f(self, &mut iter)
//        }
//    }
//
//    ///
//    #[inline]
//    pub fn seek_result_mut<R, F>(
//        &mut self,
//        op: &str,
//        key: StreamID,
//        f: F,
//    ) -> Result<R, RaxError>
//        where
//            F: Fn(
//                &mut StreamIDRax<V>,
//                &mut RaxIter<StreamID, V>,
//            ) -> Result<R, RaxError> {
//        unsafe {
//            // Allocate stack memory.
//            let mut iter: RaxIter<StreamID, V> = mem::uninitialized();
//            // Initialize a Rax iterator. This call should be performed a single time
//            // to initialize the iterator, and must be followed by a raxSeek() call,
//            // otherwise the raxPrev()/raxNext() functions will just return EOF.
//            raxStart(&iter as *const _ as *const raxIterator, self.rax);
//            iter.seek(op, key);
//            // Borrow stack iterator and execute the closure.
//            f(self, &mut iter)
//        }
//    }
}


pub struct RcRax<K: RaxKey, V> {
    pub rax: *mut rax,
    _phantom: marker::PhantomData<(K, V)>,
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
extern "C" fn Sliced_RcRax_FreeCallback<V>(v: *mut libc::c_void) {
    unsafe {
        // Decrement strong ref count
        Rc::from_raw(v as *const V);
    }
}

impl<K: RaxKey, V> Drop for RcRax<K, V> {
    fn drop(&mut self) {
        unsafe {
            println!("dropped RcMap(rax)");
            if !self.rax.is_null() {
                // Cleanup RAX
                raxFreeWithCallback(self.rax, Sliced_RcRax_FreeCallback::<V>);
            }
        }
    }
}

/// Implementation of StreamMap
impl<K: RaxKey, V> RcRax<K, V> {
    pub fn new() -> RcRax<K, V> {
        unsafe {
            RcRax {
                rax: raxNew(),
                _phantom: marker::PhantomData,
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
    pub fn insert_null(&mut self, key: &mut K) -> Result<Option<Rc<V>>, StreamError> {
        unsafe {
            // Allocate a pointer to catch the old value.
            let old: &mut *mut u8 = &mut ptr::null_mut();

            let (key_ptr, key_len) = key.into_rax();

            let r = raxInsert(
                self.rax,
                key_ptr,
                key_len,
                ptr::null_mut(),
                old,
            );

            if r == 0 && is_oom() {
                Err(StreamError::OutOfMemory)
            } else if old.is_null() {
                Ok(None)
            } else {
                // Rc the previous value since Rax is done with it and it's our
                // responsibility now to drop it. Once this Rc goes out of scope
                // the ref count is decremented and memory may be able to be reclaimed.
                Ok(Some(Rc::from_raw(*old as *const V)))
            }
        }
    }

    /// Insert a new entry into the RAX if an existing one does not exist.
    pub fn try_insert_raw(&mut self, key_ptr: *const u8, key_len: usize, data: Rc<V>) -> Result<Option<Rc<V>>, StreamError> {
        unsafe {
            // Allocate a pointer to catch the old value.
            let old: &mut *mut u8 = &mut ptr::null_mut();
            let value = Rc::into_raw(data);

            let r = raxTryInsert(
                self.rax,
                key_ptr,
                key_len,
                value as *mut V as *mut u8,
                old,
            );

            if r == 0 {
                // Decrement ref count.
                Rc::from_raw(value);

                if is_oom() {
                    Err(StreamError::OutOfMemory)
                } else {
                    // Old should never be null here. Treat as OOM.
                    if old.is_null() {
                        Err(StreamError::OutOfMemory)
                    } else {
                        // The "old" value remains in the Rax.
                        // Increment ref count.
                        Ok(Some(Rc::from_raw(old as *const _ as *const V).clone()))
                    }
                }
            } else if old.is_null() {
                Ok(None)
            } else {
                // Transfer ownership.
                Ok(Some(Rc::from_raw(*old as *const V)))
            }
        }
    }

    /// Insert a new entry into the RAX if an existing one does not exist.
    pub fn try_insert(&mut self, key: &mut K, data: Rc<V>) -> Result<Option<Rc<V>>, StreamError> {
        unsafe {
            // Allocate a pointer to catch the old value.
            let old: &mut *mut u8 = &mut ptr::null_mut();
            let value = Rc::into_raw(data);

            let (key_ptr, key_len) = key.into_rax();

            let r = raxTryInsert(
                self.rax,
                key_ptr,
                key_len,
                value as *mut V as *mut u8,
                old,
            );

            if r == 0 {
                // Decrement ref count.
                Rc::from_raw(value);

                if is_oom() {
                    Err(StreamError::OutOfMemory)
                } else {
                    // Old should never be null here. Treat as OOM.
                    if old.is_null() {
                        Err(StreamError::OutOfMemory)
                    } else {
                        // The "old" value remains in the Rax.
                        // Increment ref count.
                        Ok(Some(Rc::from_raw(old as *const _ as *const V).clone()))
                    }
                }
            } else if old.is_null() {
                Ok(None)
            } else {
                // Transfer ownership.
                Ok(Some(Rc::from_raw(*old as *const V)))
            }
        }
    }

    /// Insert a new entry into the RAX replacing and returning the existing
    /// entry for the supplied key.
    pub fn insert(&mut self, key: &mut K, data: Rc<V>) -> Result<Option<Rc<V>>, StreamError> {
        unsafe {
            // Allocate a pointer to catch the old value.
            let old: &mut *mut u8 = &mut ptr::null_mut();
            let value = Rc::into_raw(data);

            let (key_ptr, key_len) = key.into_rax();

            let r = raxInsert(
                self.rax,
                key_ptr,
                key_len,
                value as *mut V as *mut u8,
                old,
            );

            if r == 0 && is_oom() {
                // Decrement ref count.
                Rc::from_raw(value);

                Err(StreamError::OutOfMemory)
            } else if old.is_null() {
                Ok(None)
            } else {
                // Transfer ownership.
                Ok(Some(Rc::from_raw(*old as *const V)))
            }
        }
    }

    ///
    ///
    ///
    pub fn remove(&mut self, key: &mut K) -> (bool, Option<Rc<V>>) {
        unsafe {
            let old: &mut *mut u8 = &mut ptr::null_mut();

            let (key_ptr, key_len) = key.into_rax();

            let r = raxRemove(
                self.rax,
                key_ptr,
                key_len,
                old,
            );

            if old.is_null() {
                (r == 1, None)
            } else {
                // Transfer ownership.
                (r == 1, Some(Rc::from_raw(*old as *const V)))
            }
        }
    }

    ///
    ///
    ///
    pub fn find_exists(&self, key: &mut K) -> (bool, Option<Rc<V>>) {
        unsafe {
            let (key_ptr, key_len) = key.into_rax();

            let value = raxFind(
                self.rax,
                key_ptr,
                key_len,
            );

            if value.is_null() {
                (true, None)
            } else if value == raxNotFound {
                (false, None)
            } else {
                // transmute to the value so we don't drop the actual value accidentally.
                // While the key associated to the value is in the RAX then we cannot
                // drop it.
                // Transmute into Rc and increment ref count.
                (true, Some(Rc::from_raw(value as *const _ as *const V).clone()))
            }
        }
    }

    ///
    pub fn find(&self, key: &mut K) -> Option<Rc<V>> {
        unsafe {
            let (key_ptr, key_len) = key.into_rax();

            let value = raxFind(
                self.rax,
                key_ptr,
                key_len,
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
    pub fn get(&self, key: &mut K) -> Option<Rc<V>> {
        unsafe {
            let (key_ptr, key_len) = key.into_rax();

            let value = raxFind(
                self.rax,
                key_ptr,
                key_len,
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
    pub fn exists(&self, key: &mut K) -> bool {
        unsafe {
            let (key_ptr, key_len) = key.into_rax();

            let value = raxFind(
                self.rax,
                key_ptr,
                key_len,
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
    ) -> Option<K> {
        unsafe {
            // Allocate stack memory.
            let mut iter: RaxIterator<K, Rc<V>> = mem::uninitialized();
            // Initialize a Rax iterator. This call should be performed a single time
            // to initialize the iterator, and must be followed by a raxSeek() call,
            // otherwise the raxPrev()/raxNext() functions will just return EOF.
            raxStart(&iter as *const _ as *const raxIterator, self.rax);
            if raxSeek(
                &iter as *const _ as *const raxIterator,
                BEGIN.as_ptr(),
                ptr::null_mut(),
                0,
            ) != 0 || iter.key_len == 0 {
                None
            } else {
                iter.try_key()
            }
        }
    }

    ///
    #[inline]
    pub fn last_key(
        &self
    ) -> Option<K> {
        unsafe {
            // Allocate stack memory.
            let mut iter: RaxIterator<K, Rc<V>> = mem::uninitialized();
            // Initialize a Rax iterator. This call should be performed a single time
            // to initialize the iterator, and must be followed by a raxSeek() call,
            // otherwise the raxPrev()/raxNext() functions will just return EOF.
            raxStart(&iter as *const _ as *const raxIterator, self.rax);
            if raxSeek(&iter as *const _ as *const raxIterator, END.as_ptr(), ptr::null_mut(), 0) != 0 || iter.key_len == 0 {
                None
            } else {
                iter.try_key()
            }
        }
    }

    ///
    #[inline]
    pub fn seek<F>(
        &self,
        op: &str,
        key: &mut K,
        f: F,
    ) where
        F: Fn(
            &RcRax<K, V>,
            &mut RaxIterator<K, Rc<V>>,
        ) {
        unsafe {
            // Allocate stack memory.
            let mut iter: RaxIterator<K, Rc<V>> = mem::uninitialized();
            // Initialize a Rax iterator. This call should be performed a single time
            // to initialize the iterator, and must be followed by a raxSeek() call,
            // otherwise the raxPrev()/raxNext() functions will just return EOF.
            raxStart(&iter as *const _ as *const raxIterator, self.rax);
            iter.seek(op, key);
            // Borrow stack iterator and execute the closure.
            f(self, &mut iter)
        }
    }
//
//    ///
//    #[inline]
//    pub fn seek_mut<F>(
//        &mut self,
//        op: &str,
//        key: K,
//        f: F,
//    ) where
//        F: Fn(
//            &mut RcMap<K, V>,
//            &mut RaxIterator<K, V>,
//        ) {
//        unsafe {
//            // Allocate stack memory.
//            let mut iter: RaxIterator<K, V> = mem::uninitialized();
//            // Initialize a Rax iterator. This call should be performed a single time
//            // to initialize the iterator, and must be followed by a raxSeek() call,
//            // otherwise the raxPrev()/raxNext() functions will just return EOF.
//            raxStart(&iter as *const _ as *const raxIterator, self.rax);
//            iter.seek(op, key);
//            // Borrow stack iterator and execute the closure.
//            f(self, &mut iter)
//        }
//    }
//
//    ///
//    #[inline]
//    pub fn seek_result<R, F>(
//        &self,
//        op: &str,
//        key: K,
//        f: F,
//    ) -> Result<R, RaxError>
//        where
//            F: Fn(
//                &RcMap<K, V>,
//                &mut RaxIterator<K, V>,
//            ) -> Result<R, RaxError> {
//        unsafe {
//            // Allocate stack memory.
//            let mut iter: RaxIterator<K, V> = mem::uninitialized();
//            // Initialize a Rax iterator. This call should be performed a single time
//            // to initialize the iterator, and must be followed by a raxSeek() call,
//            // otherwise the raxPrev()/raxNext() functions will just return EOF.
//            raxStart(&iter as *const _ as *const raxIterator, self.rax);
//            iter.seek(op, key);
//            // Borrow stack iterator and execute the closure.
//            f(self, &mut iter)
//        }
//    }
//
//    ///
//    #[inline]
//    pub fn seek_result_mut<R, F>(
//        &mut self,
//        op: &str,
//        key: K,
//        f: F,
//    ) -> Result<R, RaxError>
//        where
//            F: Fn(
//                &mut RcMap<K, V>,
//                &mut RaxIterator<K, V>,
//            ) -> Result<R, RaxError> {
//        unsafe {
//            // Allocate stack memory.
//            let mut iter: RaxIterator<K, V> = mem::uninitialized();
//            // Initialize a Rax iterator. This call should be performed a single time
//            // to initialize the iterator, and must be followed by a raxSeek() call,
//            // otherwise the raxPrev()/raxNext() functions will just return EOF.
//            raxStart(&iter as *const _ as *const raxIterator, self.rax);
//            iter.seek(op, key);
//            // Borrow stack iterator and execute the closure.
//            f(self, &mut iter)
//        }
//    }
}


#[repr(C)]
pub struct RaxIterator<K: RaxKey, V> {
    pub flags: libc::c_int,
    pub rt: *mut rax,
    pub key: *mut u8,
    pub data: *mut libc::c_void,
    pub key_len: libc::size_t,
    pub key_max: libc::size_t,
    pub key_static_string: [u8; 128],
    pub node: *mut raxNode,
    pub stack: raxStack,
    pub node_cb: Option<raxNodeCallback>,
    _marker: std::marker::PhantomData<(K, V)>,
}


/// Core iterator implementation
impl<K: RaxKey, V> RaxIterator<K, V> {
//    pub fn new(r: RaxMap<K, V>) -> RaxIter<K, V> {
//        unsafe {
//            let mut iter: RaxIter<K, V> = mem::uninitialized();
//            raxStart(&mut iter as *mut _ as *mut raxIterator, r.rax);
//            iter
//        }
//    }

    pub fn print_ptr(&mut self) {
        println!("ptr = {:p}", self);
        println!("ptr = {:p}", self as *const _ as *const raxIterator);
    }

    #[inline]
    pub fn seek_min(&mut self) -> bool {
        unsafe {
            if raxSeek(
                self as *const _ as *const raxIterator,
                BEGIN.as_ptr(),
                ptr::null(),
                0,
            ) == 1 {
                self.forward()
            } else {
                false
            }
        }
    }

    #[inline]
    pub fn seek_max(&mut self) -> bool {
        unsafe {
            if raxSeek(
                self as *const _ as *const raxIterator,
                END.as_ptr(),
                std::ptr::null(),
                0,
            ) == 1 {
                self.back()
            } else {
                false
            }
        }
    }

    #[inline]
    pub fn back(&mut self) -> bool {
        unsafe {
            raxPrev(self as *const _ as *const raxIterator) == 1
        }
    }

    #[inline]
    pub fn forward(&mut self) -> bool {
        unsafe {
            raxNext(self as *const _ as *const raxIterator) == 1
        }
    }

    /// Key at current position
    #[inline]
    pub fn key(&self) -> K {
        K::from_rax(self.key, self.key_len as usize)
    }

    #[inline]
    pub fn try_key(&self) -> Option<K> {
        if self.key_len == 0 {
            None
        } else {
            Some(K::from_rax(self.key, self.key_len as usize))
        }
    }

    /// Data at current position.
    #[inline]
    pub fn value(&self) -> Option<&V> {
        unsafe {
            let data: *mut libc::c_void = self.data;
            if data.is_null() {
                None
            } else {
                Some(mem::transmute(data as *mut u8))
            }
        }
    }

    #[inline]
    pub fn lesser(&mut self, key: &mut K) -> bool {
        self.seek(LESSER, key)
    }

    #[inline]
    pub fn lesser_equal(&mut self, key: &mut K) -> bool {
        self.seek(LESSER_EQUAL, key)
    }

    #[inline]
    pub fn greater(&mut self, key: &mut K) -> bool {
        self.seek(GREATER, key)
    }

    #[inline]
    pub fn greater_equal(&mut self, key: &mut K) -> bool {
        self.seek(GREATER_EQUAL, key)
    }

    #[inline]
    pub fn seek(&mut self, op: &str, key: &mut K) -> bool {
        unsafe {
            let (p, len) = key.into_rax();
            raxSeek(
                self as *const _ as *const raxIterator,
                op.as_ptr(),
                p,
                len,
            ) == 1 && self.flags & RAX_ITER_EOF != 0
        }
    }

    #[inline]
    pub fn seek_raw(&mut self, op: &str, key: &mut K) -> i32 {
        unsafe {
            let (p, len) = key.into_rax();
            raxSeek(self as *const _ as *const raxIterator, op.as_ptr(), p, len)
        }
    }

    #[inline]
    pub fn seek_bytes(&mut self, op: &str, ele: &[u8]) -> bool {
        unsafe {
            raxSeek(self as *const _ as *const raxIterator, op.as_ptr(), ele.as_ptr(), ele.len() as libc::size_t) == 1
        }
    }

    /// Return if the iterator is in an EOF state. This happens when raxSeek()
    /// failed to seek an appropriate element, so that raxNext() or raxPrev()
    /// will return zero, or when an EOF condition was reached while iterating
    /// with next() and prev().
    #[inline]
    pub fn eof(&self) -> bool {
        self.flags & RAX_ITER_EOF != 0
    }
}