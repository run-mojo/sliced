use std::sync::{Arc, Weak as ArcWeak};
use std::rc::{Rc, Weak};
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::sync::atomic;
use std::ptr;

use spin::Mutex;

use ::alloc::{alloc, free, ref_counted};
use ::redis::listpack;
use ::redis::listpack::Listpack;
use ::redis::rax::{RaxMap, RaxRcMap};
use ::redis::sds::SDS;

use self::id::{next_id, StreamID};

pub mod id;
pub mod map;
pub mod raw;
pub mod aof;
pub mod record;
pub mod writer;
pub mod segment;
pub mod data_type;

pub const DEFAULT_PACK_SIZE: u32 = 65500;
// ~64KB
pub const DEFAULT_SEGMENT_SIZE: u32 = 1024 * 1024 * 64 - 64; // ~64MB

pub const COMPRESS_NONE: i32 = 0;
pub const COMPRESS_LZ4: i32 = 1;
pub const COMPRESS_ZSTD: i32 = 2;

pub struct StreamConfig {
    pub max_pack_size: u32,
    pub max_segment_size: u32,
    pub compression: i32,
}

pub const DEFAULT_CONFIG: &'static StreamConfig = &StreamConfig {
    max_pack_size: DEFAULT_PACK_SIZE, // 64KB
    max_segment_size: DEFAULT_SEGMENT_SIZE, // 1GB
    compression: COMPRESS_NONE,
};

#[derive(Debug)]
pub enum StreamError {
    OutOfMemory,
    Exists,
    WouldBlock,
    BadInput,

    Generic(String),
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "oops")
//        match *self {
//            // Both underlying errors already impl `Display`, so we defer to
//            // their implementations.
//            SlicedError::Generic(ref err) => write!(f, "{}", err),
//            SlicedError::FromUtf8(ref err) => write!(f, "{}", err),
//            SlicedError::ParseInt(ref err) => write!(f, "{}", err),
//        }
    }
}

impl Error for StreamError {
    fn description(&self) -> &str {
        match *self {
            StreamError::OutOfMemory => "Out of Memory",
            StreamError::Exists => "Exists",
            StreamError::WouldBlock => "Would Block",
            StreamError::BadInput => "Bad input",
            StreamError::Generic(ref m) => "",
            _ => "Error"
        }
    }
}

/// slice/d Stream Redis Data Type. This is the core structure. It utilizes
/// many of the Redis Streams design, models and data structures. The in-memory
/// representation at a Rax node is identical. slice/d Streams are persistent
/// and can grow as large as desired, even larger than what the local file-system
/// can hold.
///
/// A stream is broken down into files called "Segments" which have a series of
/// "Packs". The Redis data structure "Listpack" is utilized as the Record format
/// which maps one-to-one with a "Pack". The Redis Streams Listpack format is
/// kept intact which allows for very efficient interop with Redis Streams.
///
/// The I/O strategy makes generous use of memory mapped append-only files (AOF).
/// The last segment "tail" is always the file being appended to. The strategy
/// allows for writes to happen from the Redis event-loop when it's determined
/// to be non-blocking and is just a memcpy. A background I/O thread is utilized
/// for all blocking I/O operations including Creation, Truncation, Sync / Flushing,
/// Reads, Writes, etc. If the operation will block it must be scheduled on the
/// I/O thread. The initial design only utilizes a single thread for blocking
/// operations.
///
/// Memory management is very precise and very conservative. The goal is to operate
/// at RAM speeds which is inline with Redis' core tenants. Memory is allocated
/// using Redis Module memory management functions "RedisModule_Alloc",
/// "RedisModule_Realloc", and "RedisModule_Free". Redis MEMORY commands will include
/// memory allocated for slice/d Streams. That will NOT include the "on-disk" usage.
/// That is represented within some slice/d management commands.
///
/// A "max-memory" variable may be set and slice/d will try to keep the collective
/// memory usage of all streams under that amount. slice/d employs sharing the
/// same memory across any number of consumers. Once a "Pack" is loaded it is never
/// copied. Instead it is shared through a Reference Counting system and freed when
/// the "strong" count reaches 0. If no other "strong" counts exist within a
/// segment, then the segment file handle and pack index is freed as well.
///
/// slice/d utilizes the OS kernel for handling "sequential read-ahead" and will
/// take advantage of a technique the author describes as "non-blocking faulting"
/// which checks with the kernel if a Pack's on-disk pages are loaded in memory and
/// will save a background I/O request. Furthermore, since the data is shared among
/// all clients and consumers, they get a free lunch as well. Most consumers will be
/// "tailing" the stream and it would be rare to need to fault in memory in the
/// vast majority of use cases. Range queries are handled with grace all the same.
/// The bottle neck will become the throughput of Sequential file access similar
/// to Kafka.
///
/// An interesting design component of slice/d Streams comes from Redis Streams
/// record ID being a 128-bit integer epoch_ms and sequence. This allows for
/// range queries based on time practically for free. slice/d doubles down on
/// that concept and utilizes the same system for Segment files as well. Segment
/// file naming follows a simple convention. The min Record ID is used in the
/// Segment file name. By utilizing this convention, a very compact Segment index
/// can be created only utilizing the min Record ID from each segment. The minimum
/// required amount of memory is deterministic depending on the number of segments.
/// The default segment size is 64mb. ~100-150 bytes per GB (or 100kb for 1PB) is
/// needed to maintain the segment index.
pub struct Stream {
    /// Internal ID assigned when created.
    id: u64,
    /// Key string
    name: SDS,

    /// Number of bytes used within this stream.
    mem_usage: u64,
    /// The total size of all segments.
    disk_usage: u64,

    /// A segment can be in two states.
    /// 1. Not Loaded (null in rax)
    /// 2. Loaded (ptr in rax)
    segments: map::StreamIDMap<Segment>,

    /// Each stream has a single writer which has the tail segment.
    writer: Option<writer::SegmentWriter>,

    /// Configuration settings.
    config: StreamConfig,

    /// Consumer groups.
    groups: Option<RaxMap<&'static [u8], ConsumerGroup>>,
}

impl Drop for Stream {
    fn drop(&mut self) {
        // Close writer.
        match self.writer {
            Some(ref mut writer) => {}
            None => {}
        }
        self.writer = None;

        println!("dropped Stream");
    }
}

impl Stream {
    #[inline]
    pub fn last_id(&mut self) -> Option<StreamID> {
        match self.writer {
            Some(ref mut writer) => Some(writer.next_id()),
            _ => self.segments.last_key()
        }
    }
}

/// Segments contain a sequence of Packs.
/// This structure is only used when the segment is loaded/being loaded.
/// In it's unloaded state the Segment Rax in the stream will have null
/// for it's value which incurs no memory cost outside of it's ID which
/// is stored with prefix compression within the Rax. This design allows
/// for a stream to have a full index of it's keyspace sorted by segment,
/// but only load on-demand. This is ideal for Streams since most consumers
/// will be towards the tail.
pub struct Segment {
    /// A view of the segment data.
    /// The file handle is independent of the Pack index.
    data: Option<::mmap::MmapMut>,

    /// The pack index must be all inclusive of the entire segment.
    /// However, each packs data may be faulted in and freed based
    /// on demand.
    packs: map::StreamIDMap<Pack>,
}

impl Segment {
    pub fn would_block(&mut self, pack: &mut Pack) -> bool {
        // Is the pack already loaded?
        if pack.data.is_some() {
            return false
        }

        match self.data {
            Some(ref mmap) => {
                if mmap.is_resident(pack.offset as usize, pack.length as usize) {
//                    unsafe {
//                        match pack.load_segment_data(mmap.as_mut_ptr().offset(pack.offset as isize)) {}
//                    }
                    // mincore() optimization
                    // If the OS pages required to load the Pack are resident in-memory,
                    // then do load operation immediately since it won't block.

                    // Allocate listpack with room for the header.
                    let lp = ::alloc::alloc(pack.length as usize + 6);
                    unsafe {
                        ptr::copy_nonoverlapping(
                            mmap.as_ptr().offset(pack.offset as isize),
                            // Copy to right past header.
                            lp.offset(6),
                            pack.length as usize
                        );
                    }
                    // Set raw header.
                    ::redis::listpack::set_total_bytes(lp, pack.length as u32 + 6u32);
                    ::redis::listpack::set_num_elements(lp, pack.count);

                    pack.data = Some(Listpack::from_raw(lp));
                    false
                } else {
                    true
                }
            }
            None => true
        }
    }
}

impl Drop for Segment {
    fn drop(&mut self) {

        println!("dropped Segment");
    }
}

/// Packs represent a Listpack of records in standard Redis Streams format.
/// Packs can be pinned in memory to guarantee faults will not occur. This
/// is particulary important for Consumer Groups since it does not copy the
/// data for it's NACK struct in it's pending entries list (pel).
#[repr(packed)]
pub struct Pack {
    /// Keep a reference to it's parent segment to ensure the segment structure
    /// remains in memory for the lifetime of the pack.
    segment: Option<Rc<Segment>>,
    /// Offset within segment file.
    offset: u32,
    /// Number of total bytes of the listpack including it's header.
    /// This will be redundant information once a listpack is loaded since
    /// the standard Redis Streams listpack format has a 6 byte header (bytes, count).
    length: u32,
    /// Number of records inside listpack.
    /// This will be redundant information once a listpack is loaded since
    /// the standard Redis Streams listpack format has a 6 byte header (bytes, count).
    count: u16,
    /// The actual content in Redis Streams listpack format.
    /// These represent a Rax node.
    data: Option<Listpack>,
}

impl Drop for Pack {
    fn drop(&mut self) {
        // Force dealloc on the Listpack once Pack is only weakly referenced.
        self.data = None;
        // Decrement segment ref count.
        self.segment = None;

        // Debug
        println!("dropped Pack Data");
    }
}

impl Pack {
    pub fn load_segment_data(&mut self, file_lp: *mut u8) -> Result<(), StreamError> {
        // Allocate listpack with room for the header.
        let lp = alloc(self.length as usize + listpack::HDR_USIZE);
        if lp.is_null() {
            return Err(StreamError::OutOfMemory);
        }

        unsafe {
            ptr::copy_nonoverlapping(
                file_lp,
                // Copy to right past header.
                lp.offset(listpack::HDR_SIZE),
                self.length as usize
            );
        }
        // Set raw header.
        listpack::set_total_bytes(lp, self.length as u32 + listpack::HDR_USIZE as u32);
        listpack::set_num_elements(lp, self.count);

        self.data = Some(Listpack::from_raw(lp));
        Ok(())
    }
}

/// Leases and pins a pack of data into memory as well as
/// it's parent path to the stream to ensure everything stays
/// intact.
///
/// This is the essence of Streams memory management. Pins allow
/// stream operations to happen at main memory speeds with the caveat
/// that a pinned listpack may incur an I/O fault. This changes the
/// behavior into a blocking operation with a pinned future.
//pub struct Pin(Rc<Pack>);

pub type Pin = Rc<Pack>;

struct ConsumerGroup {
    /// Deduplication check.
    /// If key exists, but NACK is null then request is rejected.
    dupe: Option<RaxMap<&'static [u8], NACK>>,
    pending: RaxMap<StreamID, NACK>,

    /// Reserve the
    pins: Vec<Rc<Pack>>,
}

/// Pending (not yet acknowledged) message in a consumer group.
struct NACK {
    delivery_time: u64,
    delivery_count: u64,
    consumer: Rc<Consumer>,
    dupe: Option<SDS>,
}

struct Consumer {
    pending: RaxMap<StreamID, NACK>,
}


/// In charge of creating, reading, writing and archiving segment data.
/// Segments have an in-memory and blob representations. Blob is used for
/// both on-disk and in an object store like S3.
///
/// Redis AOF only needs to keep the root information about a stream.
/// Mainly it's name. From the name, we can use a convention for locating
/// files on both the local file-system and within an object store.
///
/// Rust's future API is used for operating on segment data away from the
/// event-loop. The segment manager receives all requests.
pub struct StreamManager {
    /// The maximum amount of RAM that is allowed for the collective
    /// usage of all Streams and all of it's in-memory representations.
    max_memory: u64,
    mem_usage: atomic::AtomicUsize,
    dir: &'static Path,
    bucket: SDS,
    streams: RaxRcMap<SDS, Stream>,
    data: Mutex<RaxMap<u64, Rc<Listpack>>>,
    pins: RaxMap<u64, Pin>,
}

impl StreamManager {
    pub fn new(bucket: SDS, path: &'static Path) -> StreamManager {
        StreamManager {
            max_memory: 0,
            mem_usage: atomic::AtomicUsize::new(0),
            dir: path.clone(),
            bucket: bucket,
            streams: RaxRcMap::new(),
            data: Mutex::new(RaxMap::new()),
            pins: RaxMap::new(),
        }
    }

    pub fn create_stream(&mut self, name: SDS) -> Result<Rc<Stream>, StreamError> {
        unsafe {
            let mut streams = &mut self.streams;

            let stream = ref_counted(Stream {
                id: 0,
                mem_usage: 0,
                disk_usage: 0,
                name: name.clone(),
                writer: None,
                segments: map::StreamIDMap::new(),
                config: StreamConfig {
                    max_pack_size: DEFAULT_PACK_SIZE,
                    max_segment_size: DEFAULT_SEGMENT_SIZE,
                    compression: COMPRESS_NONE,
                },
                groups: Some(RaxMap::new()),
            });

            match streams.try_insert(name, Rc::clone(&stream)) {
                Ok(ref existing) => {
                    if existing.is_some() {
                        return Err(StreamError::Exists)
                    }
                }
                Err(e) => {
                    match e {
                        ::redis::rax::RaxError::OutOfMemory() => return Err(StreamError::OutOfMemory),
                        _ => return Err(StreamError::Generic(String::new()))
                    }
                }
            }



            Ok(stream.clone())
        }
    }

    fn write(&mut self, stream: Rc<Stream>, id: &StreamID, record: &record::Record) {}

    fn add_segment(&mut self, mut stream: Rc<Stream>) {
        let mut s = Rc::get_mut(&mut stream).unwrap();

        let segment_id = match s.last_id() {
            Some(id) => next_id(&id),
            None => StreamID::default()
        };

        unsafe {
            let segments = &mut s.segments;

            let segment = Rc::from_raw(::alloc::leak_raw(Segment {
                data: None,
                packs: map::StreamIDMap::new(),
            }));

            // Insert into segment Rax.
            match segments.insert(&segment_id, Rc::clone(&segment)) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }

    pub fn add_pack(&self, stream: Rc<Stream>, segment: Rc<Segment>) {
//        unsafe {
//            pack = Rc::from_raw(ralloc::leak_raw(Pack {
//                id: self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
//                seq: 0,
//                offset: 0,
//                length: 0,
//                count: 0,
//                data: None,
//            }));
//        }
    }

    pub fn get_segment(&self, mut stream: Rc<Stream>, id: &StreamID) {
        let mut s = Rc::get_mut(&mut stream).unwrap();
    }

    pub fn get_pin(&self, mut stream: Rc<Stream>, id: &StreamID) {
        let mut s = Rc::get_mut(&mut stream).unwrap();
    }

    pub fn drain_queue(&self) {}
}


#[cfg(tests)]
pub mod tests {
    use super::*;

    #[test]
    pub fn construct() {
        let mut manager = StreamManager::new(
            SDS::new("mybucket"),
            std::path::Path::new("/Users/clay/sliced"),
        );
    }
}