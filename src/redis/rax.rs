#![allow(dead_code)]

use libc;

//pub struct Rax {
//    rax: *mut RedisRax
//}
//
//impl Rax {
//    pub fn new() -> Rax {
//        let rax = unsafe { raxNew() };
//        return Rax { rax };
//    }
////    pub fn length(&self) -> u32 {
////        unsafe { r(self.lp) }
////    }
//}
//
//impl Drop for Rax {
//    fn drop(&mut self) {
//        unsafe { raxFree(self.rax) }
//    }
//}
//
//#[derive(Debug, Clone, Copy)]
//#[repr(C)]
//struct RedisRax;
//
//#[allow(improper_ctypes)]
//#[allow(non_snake_case)]
//#[link(name = "redismodule", kind = "static")]
//extern "C" {
//    fn raxNew() -> *mut RedisRax;
//    fn raxFree(lp: *mut RedisRax);
//}