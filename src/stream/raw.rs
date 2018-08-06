use super::id::StreamID;
use ::redis::listpack::*;
use ::alloc::*;
use ::redis::rax::*;
use ::redis::sds::*;
use std::mem;
use std::ptr;
pub const LP_INTBUF_SIZE: usize = 21;

#[repr(C)]
pub struct stream {
    pub rax: *mut rax,
    pub length: u64,
    pub last_id: StreamID,
    pub cgroups: *mut rax,
}

impl stream {
    pub fn new() -> *mut stream {
        unsafe {
            // Heap allocate Redis Stream.
            let s = alloc(
                mem::size_of::<stream>()
            ) as *mut stream;

            (*s).rax = raxNew();
            (*s).length = 0;
            (*s).last_id.ms = 0;
            (*s).last_id.seq = 0;
            (*s).cgroups = ptr::null_mut();

            s
        }
    }
}

#[repr(C)]
pub struct streamIterator {
    pub stream: *mut stream,
    pub master_id: StreamID,
    pub master_fields_count: u64,
    pub master_fields_start: element,
    pub master_fields_ptr: element,
    pub entry_flags: i32,
    pub rev: i32,
    pub start_key: [u64; 2],
    pub end_key: [u64; 2],
    pub ri: raxIterator,
    pub lp: listpack,
    pub lp_ele: element,
    pub lp_flags: element,
    pub field_buf: [u8; LP_INTBUF_SIZE],
    pub value_buf: [u8; LP_INTBUF_SIZE],
}

#[repr(C)]
pub struct streamCG {
    pub last_id: StreamID,
    pub pel: *mut rax,
    pub consumers: *mut rax,
}

#[repr(C)]
pub struct streamNACK {
    pub delivery_time: i64,
    pub delivery_count: u64,
    pub consumer: *mut streamConsumer,
}

#[repr(C)]
pub struct streamConsumer {
    pub seen_time: i64,
    pub name: Sds,
    pub pel: *mut rax,
}

#[repr(C)]
pub struct robj {
    pub flags: i32,
    pub refcount: i32,
    pub ptr: *mut u8,
}

#[repr(C)]
pub struct streamPropInfo {
    pub keyname: *mut robj,
    pub groupname: *mut robj,

}