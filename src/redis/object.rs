use libc;

use super::sds::*;

pub const OBJ_STRING: i32 = 0;
pub const OBJ_ENCODING_RAW: i32 = 0;
pub const OBJ_ENCODING_EMBSTR: i32 = 8;

pub const OBJ_ENCODING_EMBSTR_SIZE_LIMIT: i32 = 44;

fn create_string(s: &str) -> Box<RedisString> {
    unsafe {
        let o = createStringObject(s.as_ptr() as *mut libc::c_char, s.len());
        Box::from_raw(o)
    }
}

//fn as_string(s: Sds) -> Box<RedisString> {
//    unsafe {
//        let o = createStringObject(s.as_ptr() as *mut libc::c_char, s.len());
//        Box::from_raw(o)
//    }
//}

#[repr(C)]
pub struct RedisObject {
    header: u32,
    refcount: i32,
    ptr: *mut u8,
}

#[repr(C)]
pub struct RedisString {
    header: u32,
    refcount: i32,
    ptr: Sds,
}

impl Drop for RedisString {
    fn drop(&mut self) {
        unsafe {
            crate::alloc::free(self);
            freeStringObject(self as *mut RedisString);
        }
    }
}

impl RedisString {
    #[inline]
    pub fn as_raw_sds(&self) -> Sds {
        self.ptr
    }

    #[inline]
    pub fn as_sds(&self) -> ImmutableSDS {
        ImmutableSDS(self.ptr)
    }
}

pub struct RedisModuleString {

}

impl Drop for RedisModuleString {
    fn drop(&mut self) {

    }
}



#[allow(improper_ctypes)]
#[allow(non_snake_case)]
extern "C" {
    pub fn createObject(t: libc::c_int, ptr: *mut u8) -> *mut RedisObject;
    pub fn createStringObject(ptr: *mut i8, len: libc::size_t) -> *mut RedisString;
    pub fn createRawStringObject(ptr: *mut i8, len: libc::size_t) -> *mut RedisString;
    pub fn createEmbeddedStringObject(ptr: *mut i8, len: libc::size_t) -> *mut RedisString;

    pub fn freeStringObject(o: *mut RedisString);

//    fn createObject(t: libc::c_int, ptr: *const libc::c_void) -> *mut robj;
//    fn createRawStringObject(ptr: *const libc::c_char, len: libc::size_t) -> *mut robj;
//    fn createEmbeddedStringObject(ptr: *const libc::c_char, len: libc::size_t) -> *mut robj;
//    fn makeObjectShared(ptr: *mut robj) -> *mut robj;
}