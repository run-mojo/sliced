#![allow(dead_code)]

use libc;

pub struct Stream {
    s: *mut stream,
}

impl Stream {
    pub fn new() -> Stream {
        return Stream { s: unsafe { streamNew() } };
    }
}

//
impl Drop for Stream {
    fn drop(&mut self) {
        unsafe { freeStream(self.s) }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct sds;

#[derive(Clone, Copy)]
#[repr(C)]
struct robj;

#[derive(Clone, Copy)]
#[repr(C)]
struct streamID {
    ms: libc::uint64_t,
    seq: libc::uint64_t,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct stream;

#[derive(Clone, Copy)]
#[repr(C)]
struct streamIterator;


#[allow(improper_ctypes)]
#[allow(non_snake_case)]
extern "C" {
//    fn createObject()

    fn streamNew() -> *mut stream;

    fn freeStream(s: *mut stream);

    fn streamAppendItem(s: *mut stream,
                        argv: *mut *mut libc::c_void,
                        numfields: libc::int64_t,
                        added_id: *mut streamID,
                        use_id: *mut streamID);
}

#[cfg(test)]
mod tests {
    use redis::stream::Stream;


    #[test]
    fn it_works() {
        let mut s = Stream::new();



        assert_eq!(2 + 2, 4);
    }
}