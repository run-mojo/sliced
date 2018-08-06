use ::alloc::{alloc, free, ref_counted};
use ::alloc::arc::{Arc, Weak as ArcWeak};
use ::alloc::raw_vec::RawVec;
use ::alloc::rc::{Rc, Weak};
use ::redis::listpack::Listpack;
use ::redis::rax::{RaxMap, RaxRcMap};
use ::redis::sds::SDS;
use self::id::{next_id, StreamID};
use spin::Mutex;
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::sync::atomic;

pub mod id;
pub mod map;
pub mod raw;
pub mod aof;
pub mod record;
pub mod writer;
pub mod segment;

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
            StreamError::Generic(ref m) => "",
            _ => "Error"
        }
    }
}

///
pub struct Stream {
    id: u64,
    name: SDS,
    /// Each stream has a single writer which has the tail segment.
    writer: Option<SegmentWriter>,
    /// A segment can be in two states.
    /// 1. Not Loaded (null in rax)
    /// 2. Loaded (ptr in rax)
    segments: map::StreamIDMap<Segment>,
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
struct Segment {
    /// A view of the segment data.
    data: Option<::mmap::Mmap>,

    packs: map::StreamIDMap<Pack>,
}

impl Segment {
    pub fn would_block(&self, pack: &mut Pack) -> bool {
        // Is the pack already loaded?
        if pack.data.is_some() {
            return false
        }

        match self.data {
            Some(ref mmap) => {
                // TODO: mincore() optimization
                // If the OS pages required to load the Pack are resident in-memory,
                // then do load operation immediately since it won't block.
                true
            }
            None => true
        }
    }
}

struct SegmentIndex {

}

struct AddCommand {

}

struct SegmentWriter {
    last_id: StreamID,

    /// ID of the segment is the min StreamID available within it.
    segment_id: StreamID,
    segment: Rc<Segment>,
    /// Active AOF.
    /// Path = {root_dir}/stream_id/0.dat
    aof: Option<aof::AOF>,
    tail: Rc<Pack>,

    next_segment: Rc<Segment>,
    /// Path = {root_dir}/stream_id/next.dat
    next_aof: Option<aof::AOF>,

    /// Starting segment size. This allows the ability to start all streams
    /// as compact as possible as well as optimize away truncate operations
    /// for long living streams with a history. For example, if we know we
    /// will hit the max as we have before then just allocate to the max.
    /// Segments files are relatively small so it's almost always what you
    /// want. However, this can also support many small streams such as a
    /// stream per user.
    seg_min: u32,
    /// Number of bytes to try to keep segment files within.
    seg_max: u32,

    /// Number of bytes to try to keep packs within. The larger the pack,
    /// the more compressible it could be and more records will be able
    /// to fit. A pack is the minimum sized memory allocation possible
    /// when accessing a stream. If only a single record is needed, all
    /// the other records within the Pack will be loaded as a side-effect.
    /// Since streams are meant to be well, "streamy", it's not optimized
    /// for key-value lookup although it is supported. This is optimal for
    /// range based queries though since there is great memory locality
    /// between a range of records.
    pack_max: u32,

    /// Currently waiting to here back about the grow request.
    growing: bool,
}

impl SegmentWriter {
    pub fn next_id(&mut self) -> StreamID {
        self.last_id = id::next_id(&self.last_id);
        self.last_id
    }

    pub fn finish_segment(&mut self) {
        // Write segment index to the end of it's file.

        // Rename file to the segment ID in string format "{ms}-{seq}.dat"
        // Once a file is name is changed it is guaranteed to be complete and correct.
        // If a crash happens then only the "0.dat" file in each stream needs
        // to be recovered.
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
pub struct Pack {
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
    segment: Option<Rc<Segment>>,
}

impl Drop for Pack {
    fn drop(&mut self) {
        // Free up Listpack once Pack is only weakly referenced.
        self.data = None;
        // Decrement segment ref count.
        self.segment = None;
        println!("dropped Pack Data");
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
    pins: RawVec<Pin>,
}

pub struct Cursor {
    rev: bool,
    pins: RawVec<Pin>,
}

/// Pending (not yet acknowledged) message in a consumer group.
struct NACK {
    delivery_time: u64,
    delivery_count: u64,
    consumer: Rc<Consumer>,
    dupe: SDS,
}

struct Consumer {}


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
struct StreamManager {
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

    pub fn start(&mut self) {
//        std::thread::spawn(move || {
//
//        });
    }

    pub fn create_stream(&mut self, name: SDS) -> Result<Rc<Stream>, StreamError> {
        unsafe {
            let mut streams = &mut self.streams;
//            match streams.find(name) {
//                Some(s) => return Err(StreamError::Exists)
//            }

//            if streams.exists(name.clone()) {
//                return Err(StreamError::Exists)
//            }

            let stream = ref_counted(Stream {
                id: 0,
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


//            match streams.insert_ptr(name, Rc::into_raw(stream.clone()) as *const _ as *mut u8) {
//                Ok(_) => {}
//                Err(e) => {
//                    match e {
//                        RaxError::OutOfMemory() => return Err(StreamError::OutOfMemory),
//                        _ => return Err(StreamError::Generic(String::new()))
//                    }
//                }
//            }

            // Write to global AOF + Propagate
            // MO.X CREATE mystream id 10 seg 64mb pack 64kb
            // MO.X GROUP mystream mygroup
            // MO.X DELGROUP mystream mygroup
            // MO.X CONSUMER
            // MO.X DELCONSUMER
            // MO.X FAILED mystream mygroup id 10
            // MO.X SEG mystream 10
            // MO.X SEGMERGE mystream 10 11
            // MO.X SEGDEL mystream 10
            // MO.X DEL mystream
            // MO.X SEGMENT CREATE 10 10000003832-10
            // MO.XADD
            // MO.X SEGMENT FOLD 10000002340-10


            // Rewrite AOF
            // Save stream

            // MO.X SEGMENT ADD mystream 10 start 64mb max 64mb pack 64kb

            // MO.STREAM REMOVE SEGMENT mystream 10

            // MO.XADD
            // MO.XREADGROUP
            // MO.XREAD
            // MO.XRANGE
            // MO.XTRIM
            // MO.XDEL

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