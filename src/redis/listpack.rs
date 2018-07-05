#![allow(dead_code)]

use dlopen::wrapper::{Container, WrapperApi};
use libc;

//static API: ::RedisApi = ::REDIS_EXE;

//static API: Container<::RedisApi> = ::REDIS_API.lpLength;
const LP_INTBUF_SIZE: libc::c_int = 21;

const LP_BEFORE: libc::c_int = 0;
const LP_AFTER: libc::c_int = 1;
const LP_REPLACE: libc::c_int = 2;

pub enum Where {
    Before = 0,
    After = 1,
    Replace = 2,
}


pub struct ListPack {
    lp: *mut u8
}

impl ListPack {
    pub fn is_installed(&self) {}

    pub fn new() -> ListPack {
        return ListPack { lp: unsafe { lpNew() } };
    }

    pub fn length(&self) -> u32 {
        unsafe { lpLength(self.lp) as u32 }
    }

    pub fn insert_str(&mut self, s: &str, p: *mut u8, w: Where, newp: *mut *mut u8) {
        self.insert(s.as_ptr(), s.len() as u32, p, w, newp);
    }

    pub fn insert_bytes(&mut self,
                        ele: &mut [u8],
                        p: *mut u8,
                        w: Where,
                        newp: *mut *mut u8) {
        self.lp = unsafe {
            lpInsert(self.lp,
                     ele.as_mut_ptr(),
                     ele.len() as libc::uint32_t,
                     p,
                     w as libc::c_int,
                     newp,
            )
        };
    }

    pub fn insert(&mut self,
                  ele: *const u8,
                  size: u32,
                  p: *mut u8,
                  w: Where,
                  newp: *mut *mut u8) {
        self.lp = unsafe {
            lpInsert(
                self.lp,
                ele,
                size as libc::uint32_t,
                p,
                w as libc::c_int,
                newp,
            )
        };
    }

    pub fn append_bytes(&mut self,
                        ele: &mut [u8]) {
        self.lp = unsafe {
            lpAppend(self.lp,
                     ele.as_mut_ptr(),
                     ele.len() as libc::uint32_t)
        };
    }

    pub fn append(&mut self,
                  ele: *const u8,
                  size: u32) {
        self.lp = unsafe { lpAppend(self.lp, ele, size) };
    }

    pub fn delete() {}

    pub fn get() {}

    pub fn first() {}

    pub fn last() {}

    pub fn next() {}

    pub fn prev() {}

    pub fn bytes() {}

    pub fn seek() {}
}

// Map Drop -> "lpFree"
impl Drop for ListPack {
    fn drop(&mut self) {
        unsafe { lpFree(self.lp) }
    }
}

#[allow(improper_ctypes)]
#[allow(non_snake_case)]
extern "C" {
    fn lpNew() -> *mut u8;

    fn lpFree(lp: *mut u8);

    fn lpInsert(lp: *mut u8,
                ele: *const u8,
                size: libc::uint32_t,
                p: *mut u8,
                wh: libc::c_int,
                newp: *mut *mut u8) -> *mut u8;

    fn lpAppend(lp: *mut u8,
                ele: *const u8,
                size: libc::uint32_t) -> *mut u8;

    fn lpDelete(lp: *mut u8,
                p: *mut u8,
                newp: *mut *mut u8) -> *mut u8;

    fn lpLength(lp: *mut u8) -> libc::uint32_t;

    fn lpGet(lp: *mut u8, count: *mut libc::int64_t, intbuf: *mut u8) -> *mut u8;
    fn lpFirst(lp: *mut u8) -> *mut u8;
    fn lpLast(lp: *mut u8) -> *mut u8;
    fn lpNext(lp: *mut u8, p: *mut u8) -> *mut u8;
    fn lpPrev(lp: *mut u8, p: *mut u8) -> *mut u8;

    fn lpBytes(lp: *mut u8) -> libc::uint32_t;
    fn lpSeek(lp: *mut u8, index: libc::c_long) -> *mut u8;
}


#[cfg(test)]
mod tests {
    extern crate libc;

    use libc::{c_char, c_int};
    use redis::listpack;
    use redis::listpack::ListPack;
    use std::env;
    use std::ffi::CStr;


    #[test]
    fn it_works() {
        let mut lp = ListPack::new();


        assert_eq!(2 + 2, 4);
    }
}