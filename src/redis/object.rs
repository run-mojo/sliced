use libc;

fn create_object() {}

#[derive(Clone, Copy)]
#[repr(C)]
struct robj;

#[allow(improper_ctypes)]
#[allow(non_snake_case)]
extern "C" {
//    fn createObject(t: libc::c_int, ptr: *const libc::c_void) -> *mut robj;
//    fn createRawStringObject(ptr: *const libc::c_char, len: libc::size_t) -> *mut robj;
//    fn createEmbeddedStringObject(ptr: *const libc::c_char, len: libc::size_t) -> *mut robj;
//    fn makeObjectShared(ptr: *mut robj) -> *mut robj;
}