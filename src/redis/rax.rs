#![allow(dead_code)]

use libc;

pub struct Rax {
    rax: *mut rax,
}

impl Rax {
    pub fn new() -> Rax {
        return Rax { rax: unsafe { raxNew() } };
    }
}

//
impl Drop for Rax {
    fn drop(&mut self) {
        unsafe { raxFree(self.rax) }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct rax;

#[derive(Clone, Copy)]
#[repr(C)]
struct raxIterator;


#[allow(improper_ctypes)]
#[allow(non_snake_case)]
extern "C" {
    fn raxNew() -> *mut rax;

    fn raxFree(rax: *mut rax);

    fn raxInsert(rax: *mut rax,
                 s: *mut u8,
                 len: libc::size_t,
                 data: *mut libc::c_void,
                 old: *mut *mut libc::c_void) -> libc::c_int;

    fn raxTryInsert(rax: *mut rax,
                    s: *mut u8,
                    len: libc::size_t,
                    data: *mut libc::c_void,
                    old: *mut *mut libc::c_void) -> libc::c_int;

    fn raxRemove(rax: *mut rax,
                 s: *mut u8,
                 len: libc::size_t,
                 old: *mut *mut libc::c_void) -> libc::c_int;
}