extern crate sliced;
extern crate time;

use sliced::mmap::*;
use sliced::redis::listpack::*;
use sliced::redis::rax::*;
use std::cmp::Ordering;
use std::fmt;

fn main() {
    let mut s = Stream::new();
    let mut size: usize = 1024 * 2;
    println!("PageSize: {}", sliced::page_size::get());

    let mut record_id = StreamID { ms: 0, seq: 0 };

    for i in 0..10 {
//        println!("{}", size.next_power_of_two());
//        size = size + 1;
//        size = size.next_power_of_two();


        record_id = next_id(&record_id);
        let record = Record {};
        s.put_record(record_id, &record);
    }
}

pub static mut WRITER_MAP_SIZE: usize = 1024 * 1024;

// Redis Streams entry flags
pub const STREAM_ITEM_FLAG_NONE: i32 = 0;               /* No special flags. */
pub const STREAM_ITEM_FLAG_DELETED: i32 = (1 << 0);     /* Entry is delted. Skip it. */
pub const STREAM_ITEM_FLAG_SAMEFIELDS: i32 = (1 << 1);  /* Same fields as master entry. */
pub const STREAM_ITEM_FLAG_SLOT: i32 = (1 << 2);        /* Has slot number */
pub const STREAM_ITEM_FLAG_TX: i32 = (1 << 3);          /* Has tx key */
pub const STREAM_ITEM_FLAG_DEDUPE: i32 = (1 << 4);      /* Has de-duplication key */

/// Reserved field name for Slot number chosen.
pub const FIELD_SLOT: &'static [u8] = b"*";
//pub const FIELD_SLOT: &'static str = "*";
pub const FIELD_TX_KEY: &'static str = "^";
pub const FIELD_CALLER_ID: &'static str = "#";
pub const FIELD_REPLY_MAILBOX: &'static str = "@";
pub const FIELD_DUPE_KEY: &'static str = "?";
pub const FIELD_DEFER: &'static str = "!";

pub type PageID = u64;
pub type RecordID = StreamID;

/// Each listpack conforms to a particular memory layout or format.
/// The protocol trait serves that very purpose for translating the
/// elements of a listpack into something meaningful.
pub trait Protocol<H, T> where H: Sized, T: Sized {
    fn header(lp: listpack) -> Option<H>;
    fn read(lp: listpack, pos: element) -> Option<T>;
}

pub struct Record {}

struct Stream {
    /// The last segment that new blocks are appended too.
    /// The memory is managed via RaxMap
    tail: *mut Segment,
    segments: RaxMap<RecordID, Segment>,
}

struct Segment {
    id: RecordID,
    flags: i32,
    tail: *mut Block,
    blocks: RaxMap<RecordID, Block>,
    file: Option<SegFile>,
}

enum SegFile {
    Writer(std::fs::File, MmapMut),
    Reader(std::fs::File, Mmap),
}

trait SegmentFile {
    fn is_writable() -> bool;

    fn append(&mut self, buf: *mut u8, size: usize) {}
}

struct SegmentWriter {
    file: std::fs::File,
    mmap: MmapMut,
}

struct SegmentReader {
    mmap: Mmap,
}


pub struct Block {
    seq: u64,
    offset: u64,
    flags: i32,
    records: Listpack,
}

pub struct BlockLocation {
    segment: u32,
    offset: u64,
    len: u32,
}

impl Stream {
    pub fn new() -> Stream {
        let s = Stream {
            segments: RaxMap::new(),
            tail: std::ptr::null_mut(),
        };
        s
    }

    pub fn put_record(&mut self, id: RecordID, record: &Record) {
        // Does a tail segment exist.
        if self.tail.is_null() {
            let segments = &mut (self.segments);
            let mut seg: &mut Segment = Box::leak(Box::new(Segment {
                id,
                flags: 0,
                tail: std::ptr::null_mut(),
                blocks: RaxMap::new(),
                file: None,
            }));
            unsafe {
                seg.append(id, record);
                self.tail = seg as *const _ as *mut Segment;
                match segments.insert_ptr(id, self.tail as *mut u8) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        } else {
            unsafe {
                (&mut (*self.tail)).append(id, record);
            }
        }
    }

    pub fn get_block(&self, id: RecordID) {
        // Locate segment.
//            self.segments.seek(GREATER_EQUAL, id, |m,iter| {
//                // Locate block.
//            });
    }

    pub fn is_block_loaded(&self, id: RecordID) {}
}

impl Segment {
    pub fn append(&mut self, id: RecordID, record: &Record) {
        println!("Segment::append -> {}", id);
        if self.tail.is_null() {
            let blocks = &mut (self.blocks);
            let mut block: &mut Block = Box::leak(Box::new(Block {
                seq: blocks.len() as u64,
                offset: 0,
                flags: 0,
                records: Listpack::new(),
            }));
        }
    }
}

impl SegmentWriter {
//    pub fn new(file: &std::fs::File) -> std::io::Result<SegmentWriter> {
//        unsafe {
//            match memmap::MmapOptions::new().map_mut(file) {
//                Ok(mmap) => {
//                    Ok(SegmentWriter {
//                        mmap
//                    })
//                }
//                Err(e) => {
//                    Err(e)
//                }
//            }
//        }
//    }

    pub fn grow(&mut self) {}
}

impl Drop for SegmentWriter {
    fn drop(&mut self) {
        println!("dropped SegmentWriter");
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        println!("dropped Stream");
    }
}

impl Drop for Segment {
    fn drop(&mut self) {
        println!("dropped Segment");
    }
}


#[derive(Copy)]
#[repr(C)]
pub struct StreamID {
    pub ms: u64,
    pub seq: u64,
}

impl fmt::Debug for StreamID {
    fn fmt(&self, _f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl fmt::Display for StreamID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}-{}", self.ms, self.seq);
        Ok(())
    }
}

impl Default for StreamID {
    fn default() -> Self {
        StreamID { ms: 0, seq: 0 }
    }
}

impl Clone for StreamID {
    fn clone(&self) -> Self {
        StreamID { ms: self.ms, seq: self.seq }
    }
}

impl PartialEq for StreamID {
    fn eq(&self, other: &StreamID) -> bool {
        self.ms == other.ms && self.seq == other.seq
    }
}

impl PartialOrd for StreamID {
    fn partial_cmp(&self, other: &StreamID) -> Option<Ordering> {
        if self.ms > other.ms {
            Some(Ordering::Greater)
        } else if self.ms < other.ms {
            Some(Ordering::Less)
        } else if self.seq > other.seq {
            Some(Ordering::Greater)
        } else if self.seq < other.seq {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl RaxKey for StreamID {
    type Output = StreamID;

    #[inline]
    fn encode(self) -> Self::Output {
        StreamID {
            ms: self.ms.to_be(),
            seq: self.seq.to_be(),
        }
    }

    #[inline]
    fn to_buf(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, std::mem::size_of::<StreamID>())
    }

    #[inline]
    fn from_buf(ptr: *const u8, len: usize) -> StreamID {
        if len != std::mem::size_of::<StreamID>() {
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

#[inline(always)]
fn mstime() -> u64 {
    time::precise_time_ns() / 1000000
}

/// Generate the next stream item ID given the previous one. If the current
/// milliseconds Unix time is greater than the previous one, just use this
/// as time part and start with sequence part of zero. Otherwise we use the
/// previous time (and never go backward) and increment the sequence.
#[inline(always)]
fn next_id(last: &StreamID) -> StreamID {
    let ms = mstime();
    if ms > last.ms {
        StreamID { ms, seq: 0 }
    } else {
        StreamID { ms: last.ms, seq: last.seq + 1 }
    }
}
