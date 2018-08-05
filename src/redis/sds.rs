#![allow(dead_code)]
#![feature(lang_items)]
#![feature(test)]

use libc;

use std;
use std::cmp::Ordering;
use std::fmt;

const SDS_TYPE_5: libc::c_char = 0;
const SDS_TYPE_8: libc::c_char = 1;
const SDS_TYPE_16: libc::c_char = 2;
const SDS_TYPE_32: libc::c_char = 3;
const SDS_TYPE_64: libc::c_char = 4;
const SDS_LLSTR_SIZE: libc::c_int = 21;
const SDS_TYPE_MASK: libc::c_int = 7;
const SDS_TYPE_BITS: libc::c_int = 3;

pub type Sds = *mut libc::c_char;

#[derive(Eq, Debug)]
#[repr(C)]
pub struct SDS(pub Sds);

unsafe impl Send for SDS {}
unsafe impl Sync for SDS {}

impl Clone for SDS {
    fn clone(&self) -> Self {
        SDS(dup(self.0))
    }
}

impl Default for SDS {
    fn default() -> Self {
        return SDS(empty());
    }
}

impl PartialEq for SDS {
    fn eq(&self, other: &SDS) -> bool {
        unsafe { sdscmp(self.0, other.0) == 0 }
    }

    fn ne(&self, other: &SDS) -> bool {
        unsafe { sdscmp(self.0, other.0) != 0 }
    }
}

impl PartialOrd for SDS {
    fn partial_cmp(&self, other: &SDS) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SDS {
    fn cmp(&self, other: &Self) -> Ordering {
        match unsafe { sdscmp(self.0, other.0) } {
            -1 => Ordering::Less,
            0 => Ordering::Equal,
            1 => Ordering::Greater,
            _ => Ordering::Greater
        }
    }
}

impl fmt::Display for SDS {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(self.to_str()).expect("");
        Ok(())
    }
}

impl Drop for SDS {
    fn drop(&mut self) {
        free(self.0)
    }
}

impl SDS {
    #[inline]
    pub fn empty() -> SDS {
        SDS(empty())
    }

    #[inline]
    pub fn new(s: &str) -> SDS {
        SDS(new_len(s))
    }

    #[inline]
    pub fn from_ptr(init: *const u8, initlen: usize) -> SDS {
        unsafe { SDS(sdsnewlen(init, initlen)) }
    }

    #[inline]
    pub fn from_cstr(s: *const u8) -> SDS {
        unsafe { SDS(sdsnew(s)) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        get_len(self.0)
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.0 as *const u8
    }

    #[inline]
    pub fn avail(&self) -> usize {
        avail(self.0)
    }

    #[inline]
    pub fn hdr_size(&self) -> libc::c_int {
        hdr_size(self.0)
    }

    /// Return the total size of the allocation of the specified sds string,
    /// including:
    /// 1) The sds header before the pointer.
    /// 2) The string.
    /// 3) The free buffer at the end if any.
    /// 4) The implicit null term.
    #[inline]
    pub fn alloc_size(&self) -> usize {
        alloc_size(self.0)
    }

    /// Return the pointer of the actual SDS allocation (normally SDS strings
    /// are referenced by the start of the string buffer).
    #[inline]
    pub fn alloc_ptr(&self) -> *mut libc::c_void {
        alloc_ptr(self.0)
    }


    #[inline]
    pub fn lower(&mut self) {
        to_lower(self.0)
    }

    #[inline]
    pub fn upper(&mut self) {
        to_upper(self.0)
    }

    #[inline]
    pub fn to_str<'a>(&self) -> &'a str {
        unsafe {
            std::str::from_utf8(
                std::slice::from_raw_parts(
                    self.0 as *const u8,
                    self.len())
            ).unwrap()
        }
    }

    #[inline]
    pub fn sds_type(&self) -> libc::c_int {
        get_type(self.0)
    }

    /// Modify an sds string in-place to make it empty (zero length).
    /// However all the existing buffer is not discarded but set as free space
    /// so that next append operations will not require allocations up to the
    /// number of bytes previously available.
    #[inline]
    pub fn clear(&mut self) {
        clear(self.0)
    }

    /// Enlarge the free space at the end of the sds string so that the caller
    /// is sure that after calling this function can overwrite up to addlen
    /// bytes after the end of the string, plus one more byte for nul term.
    ///
    /// Note: this does not change the *length* of the sds string as returned
    /// by sdslen(), but only the free buffer space we have.
    #[inline]
    pub fn extend(&mut self, addlen: libc::size_t) {
        self.0 = make_room_for(self.0, addlen)
    }

    /// Reallocate the sds string so that it has no free space at the end. The
    /// contained string remains not altered, but next concatenation operations
    /// will require a reallocation.
    ///
    /// After the call, the passed sds string is no longer valid and all the
    /// references must be substituted with the new pointer returned by the call.
    #[inline]
    pub fn compact(&mut self) {
        self.0 = remove_free_space(self.0)
    }

    /// Increment the sds length and decrements the left free space at the
    /// end of the string according to 'incr'. Also set the null term
    /// in the new end of the string.
    ///
    /// This function is used in order to fix the string length after the
    /// user calls sdsMakeRoomFor(), writes something after the end of
    /// the current string, and finally needs to set the new length.
    ///
    /// Note: it is possible to use a negative increment in order to
    /// right-trim the string.
    ///
    /// Usage example:
    ///
    /// Using sdsIncrLen() and sdsMakeRoomFor() it is possible to mount the
    /// following schema, to cat bytes coming from the kernel to the end of an
    /// sds string without copying into an intermediate buffer:
    ///
    /// oldlen = sdslen(s);
    /// s = sdsMakeRoomFor(s, BUFFER_SIZE);
    /// nread = read(fd, s+oldlen, BUFFER_SIZE);
    /// ... check for nread <= 0 and handle it ...
    /// sdsIncrLen(s, nread);
    #[inline]
    pub fn incr_len(&mut self, incr: libc::ssize_t) -> bool {
        let new_len = self.len() + incr as usize;
        if new_len > self.alloc_size() {
            return false;
        }
        incr_len(self.0, incr);
        true
    }

    #[inline]
    pub fn incr_len_dangerously(&mut self, incr: libc::ssize_t) -> bool {
        incr_len(self.0, incr);
        true
    }
}

#[inline]
pub fn new_raw(s: *const u8, len: usize) -> Sds {
    unsafe { sdsnewlen(s, len) }
}

/// Create a new sds string with the content specified by the 'init' pointer
/// and 'initlen'.
/// If NULL is used for 'init' the string is initialized with zero bytes.
/// If SDS_NOINIT is used, the buffer is left uninitialized;
///
/// The string is always null-termined (all the sds strings are, always) so
/// even if you create an sds string with:
///
/// mystring = sdsnewlen("abc",3);
///
/// You can print the string with printf() as there is an implicit \0 at the
/// end of the string. However the string is binary safe and can contain
/// \0 characters in the middle, as the length is stored in the sds header.
#[inline]
pub fn new_len(s: &str) -> Sds {
    unsafe { sdsnewlen(s.as_ptr(), s.len()) }
}

/// Create a new sds string starting from a null terminated C string.
#[inline]
pub fn new(s: &str) -> Sds {
//    unsafe { sdsnew(format!("{}\0", s).as_ptr()) }
    unsafe { sdsnewlen(s.as_ptr(), s.len()) }
}

#[inline]
pub fn from_long_long(value: libc::c_longlong) -> Sds {
    unsafe { sdsfromlonglong(value) }
}

/// Create an empty (zero length) sds string. Even in this case the string
/// always has an implicit null term.
#[inline]
pub fn empty() -> Sds {
    unsafe { sdsempty() }
}

/// Free an sds string. No operation is performed if 's' is NULL.
#[inline]
pub fn free(s: Sds) {
    unsafe { sdsfree(s) }
}

/// Duplicate an sds string.
#[inline]
pub fn dup(s: Sds) -> Sds {
    unsafe { sdsdup(s) }
}

/// Modify an sds string in-place to make it empty (zero length).
/// However all the existing buffer is not discarded but set as free space
/// so that next append operations will not require allocations up to the
/// number of bytes previously available.
#[inline]
pub fn clear(s: Sds) {
    unsafe { sdsclear(s) }
}

/// Enlarge the free space at the end of the sds string so that the caller
/// is sure that after calling this function can overwrite up to addlen
/// bytes after the end of the string, plus one more byte for nul term.
///
/// Note: this does not change the *length* of the sds string as returned
/// by sdslen(), but only the free buffer space we have.
#[inline]
pub fn make_room_for(s: Sds, addlen: libc::size_t) -> Sds {
    unsafe { sdsMakeRoomFor(s, addlen) }
}

/// Reallocate the sds string so that it has no free space at the end. The
/// contained string remains not altered, but next concatenation operations
/// will require a reallocation.
///
/// After the call, the passed sds string is no longer valid and all the
/// references must be substituted with the new pointer returned by the call.
#[inline]
pub fn remove_free_space(s: Sds) -> Sds {
    unsafe { sdsRemoveFreeSpace(s) }
}

/// Increment the sds length and decrements the left free space at the
/// end of the string according to 'incr'. Also set the null term
/// in the new end of the string.
///
/// This function is used in order to fix the string length after the
/// user calls sdsMakeRoomFor(), writes something after the end of
/// the current string, and finally needs to set the new length.
///
/// Note: it is possible to use a negative increment in order to
/// right-trim the string.
///
/// Usage example:
///
/// Using sdsIncrLen() and sdsMakeRoomFor() it is possible to mount the
/// following schema, to cat bytes coming from the kernel to the end of an
/// sds string without copying into an intermediate buffer:
///
/// oldlen = sdslen(s);
/// s = sdsMakeRoomFor(s, BUFFER_SIZE);
/// nread = read(fd, s+oldlen, BUFFER_SIZE);
/// ... check for nread <= 0 and handle it ...
/// sdsIncrLen(s, nread);
#[inline]
pub fn incr_len(s: Sds, incr: libc::ssize_t) {
    unsafe { sdsIncrLen(s, incr) }
}

/// Compare two sds strings s1 and s2 with memcmp().
///
/// Return value:
///
///     positive if s1 > s2.
///     negative if s1 < s2.
///     0 if s1 and s2 are exactly the same binary string.
///
/// If two strings share exactly the same prefix, but one of the two has
/// additional characters, the longer string is considered to be greater than
/// the smaller one.
#[inline]
pub fn cmp(s1: Sds, s2: Sds) -> libc::c_int {
    unsafe { sdscmp(s1, s2) }
}

#[inline]
pub fn get_len(s: Sds) -> libc::size_t {
    unsafe { sds_getlen(s) }
}

#[inline]
pub fn avail(s: Sds) -> libc::size_t {
    unsafe { sds_avail(s) }
}

#[inline]
pub fn grow_zero(s: Sds, len: libc::size_t) -> Sds {
    unsafe { sdsgrowzero(s, len) }
}


#[inline]
pub fn to_lower(s: Sds) {
    unsafe { sdstolower(s) }
}

#[inline]
pub fn to_upper(s: Sds) {
    unsafe { sdstoupper(s) }
}

/// Return the total size of the allocation of the specifed sds string,
/// including:
/// 1) The sds header before the pointer.
/// 2) The string.
/// 3) The free buffer at the end if any.
/// 4) The implicit null term.
#[inline]
pub fn alloc_size(s: Sds) -> libc::size_t {
    unsafe { sdsAllocSize(s) }
}

/// Return the pointer of the actual SDS allocation (normally SDS strings
/// are referenced by the start of the string buffer).
#[inline]
pub fn alloc_ptr(s: Sds) -> *mut libc::c_void {
    unsafe { sdsAllocPtr(s) }
}

#[inline]
pub fn get_type(s: Sds) -> libc::c_int {
    unsafe { sds_type(s) }
}

#[inline]
pub fn hdr_size_for(t: libc::c_char) -> libc::c_int {
    unsafe { sds_hdr_size(t) }
}

#[inline]
pub fn hdr_size(s: Sds) -> libc::c_int {
    unsafe { sds_get_hdr_size(s) }
}

#[link(name = "redismodule", kind = "static")]
extern "C" {
    #[no_mangle]
    pub static SDS_NOINIT: *mut libc::c_char;

    pub fn sdsnewlen(init: *const u8, initlen: libc::size_t) -> Sds;

    fn sdsnew(init: *const u8) -> Sds;

    fn sdsempty() -> Sds;

    fn sdsfree(s: Sds);

    fn sdsfromlonglong(value: libc::c_longlong) -> Sds;

    fn sdsdup(s: Sds) -> Sds;

    fn sdsclear(s: Sds);

    fn sdsMakeRoomFor(s: Sds, addlen: libc::size_t) -> Sds;

    fn sdsRemoveFreeSpace(s: Sds) -> Sds;

    fn sdsIncrLen(s: Sds, incr: libc::ssize_t);

    fn sdscmp(s1: Sds, s2: Sds) -> libc::c_int;

    fn sds_getlen(s: Sds) -> libc::size_t;

    fn sds_avail(s: Sds) -> libc::size_t;

    fn sdsgrowzero(s: Sds, len: libc::size_t) -> Sds;

    fn sdstolower(s: Sds);

    fn sdstoupper(s: Sds);

    fn sds_hdr_size(t: libc::c_char) -> libc::c_int;

    fn sds_get_hdr_size(s: Sds) -> libc::c_int;

    fn sds_type(s: Sds) -> libc::c_int;

    fn sdsAllocSize(s: Sds) -> libc::size_t;

    fn sdsAllocPtr(s: Sds) -> *mut libc::c_void;

//    fn sdsll2str(s: )
}

#[cfg(test)]
mod tests {
    use redis::sds::*;
    use std::cmp::Ordering;

    #[test]
    fn test_cmp() {
        let s1 = SDS::new("test");
        let s2 = SDS::new("test1");

        assert_eq!(s1.cmp(&s2), Ordering::Less);
        assert_eq!(s2.cmp(&s1), Ordering::Greater);
        assert_eq!(SDS::new("test").cmp(&s1), Ordering::Equal);
    }

    #[test]
    fn test_len() {
        println!("{}", std::mem::size_of::<SDS>());
        println!("{}", std::mem::size_of::<Sds>());

        let mut ss = SDS::new("hello");

        println!("HDR Size: {}", ss.hdr_size());

        println!("HDR5 Size: {}", hdr_size_for(SDS_TYPE_5));
        println!("HDR8 Size: {}", hdr_size_for(SDS_TYPE_8));
        println!("HDR16 Size: {}", hdr_size_for(SDS_TYPE_16));
        println!("HDR32 Size: {}", hdr_size_for(SDS_TYPE_32));
        println!("HDR64 Size: {}", hdr_size_for(SDS_TYPE_64));


//        println!("Alloc Size: {}", sds::alloc_size(s));
//        println!("Length    : {}", sds::len(s));
//        println!("Avail Size: {}", sds::avail(s));
//        sds::incr_len(s, 10);
//        println!("Length    : {}", sds::len(s));
//        println!("Avail Size: {}", sds::avail(s));


        println!("Alloc Size: {}", ss.alloc_size());
        println!("Length    : {}", ss.len());
        println!("Avail Size: {}", ss.avail());

        ss.extend(1);
        println!("Alloc Size: {}", ss.alloc_size());

//        ss.compact();
//        println!("Alloc Size: {}", ss.alloc_size());

        ss.incr_len(1);
        println!("Alloc Size: {}", ss.alloc_size());
        println!("Length    : {}", ss.len());
        println!("Avail Size: {}", ss.avail());

        println!("HDR Size: {}", ss.hdr_size());

//        unsafe {sds::sdslength(s);}
//        assert_eq!(sds::sds_len(s), 5);
    }
}