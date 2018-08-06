extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate futures;
extern crate rand;
extern crate sliced;
extern crate spin;
extern crate tempdir;
extern crate time;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_current_thread;
extern crate tokio_executor;
extern crate tokio_fs;
extern crate tokio_io;
extern crate tokio_reactor;
extern crate tokio_threadpool;
extern crate tokio_timer;

use futures::Future;
use futures::future::Map;
use futures::future::poll_fn;
use futures::prelude::*;
use futures::sync::oneshot;
use rand::{Rng, thread_rng};
use sliced::mmap::*;
use sliced::mmap::MmapMut;
use sliced::page_size;
use sliced::redis::listpack::*;
use sliced::redis::listpack::*;
use sliced::alloc::*;
use sliced::alloc::rc::Rc;
use sliced::redis::rax::*;
use sliced::redis::sds::*;

use spin::RwLock;
use std::cmp::Ordering;
use std::fmt;
use std::fs::File;
use std::io::{Read, SeekFrom};
use std::io::Error as IoError;
use std::io::Result as IoResult;
use std::mem;
use std::ptr::null_mut;
use std::sync::Arc;
use std::sync::atomic;
use std::sync::atomic::AtomicUsize;
use std::time::{Duration, Instant};
use tempdir::TempDir;
use tokio_fs::*;
use tokio_io::io;
use tokio_threadpool::*;


fn main() {
    let mut manager = StreamManager::new(
        SDS::new("mybucket"),
        std::path::Path::new("/Users/clay/sliced")
    );
    let mut size: usize = 1024 * 2;
    println!("PageSize: {}", sliced::page_size::get());
//    println!("RedisModule_Alloc -> {}", sliced::redis::api::RedisModule_Alloc);

    let mut record_id = StreamID { ms: 0, seq: 0 };

    for i in 0..10 {
//        println!("{}", size.next_power_of_two());
//        size = size + 1;
//        size = size.next_power_of_two();

        record_id = next_id(&record_id);
//        let record = Record {};
//        s.append(&mut record_id, &record);
    }
}

fn main1() {
    use std::sync::Arc;
    use std::thread;

    let five = Arc::new(5);

    let mut v:Vec<std::thread::JoinHandle<()>> = vec![];

    for _ in 0..10 {
        let five = Arc::clone(&five);


        v.push(thread::spawn(move || {
            println!("{:?}", five);
//            println!("{}", five);
            println!("Ref Count: {}", Arc::strong_count(&five));
        }));
    }

    for a in v {
        a.join();
    }

    println!("Ref Count: {}", Arc::strong_count(&five));
}


//fn main2() {
//    let dir = TempDir::new("tokio-fs-tests").unwrap();
//    let file_path = dir.path().join("seek.txt");
//
//    let pool = Builder::new().pool_size(2).max_blocking(2).build();
//    let (tx, rx) = oneshot::channel();
//
//    pool.spawn(
//        OpenOptions::new()
//            .create(true)
//            .read(true)
//            .write(true)
//            .open(file_path)
//            .and_then(|file| {
//                println!("opened file");
//                Ok(file)
//            })
//            .and_then(|file| {
//                println!("writing...");
//                io::write_all(file, "Hello, world!")
//            })
//            .and_then(|(file, _)| {
//                println!("seeking...");
//                file.seek(SeekFrom::End(-6))
//            })
//            .and_then(|(file, _)| {
//                println!("reading...");
//                io::read_exact(file, vec![0; 5])
//            })
//            .and_then(|(file, buf)| {
//                assert_eq!(buf, b"world");
//                file.seek(SeekFrom::Start(0))
//            })
//            .and_then(|(file, _)| io::read_exact(file, vec![0; 5]))
//            .and_then(|(_, buf)| {
//                assert_eq!(buf, b"Hello");
//                Ok(())
//            })
//            .then(|r| {
//                match r {
//                    Ok(rr) => {
//                        let _ = r.unwrap();
//                        tx.send(())
//                    }
//                    Err(e) => {
//                        match e.kind() {
//                            std::io::ErrorKind::NotFound => {
//                                println!("not found")
//                            }
//                            _ => {
//                                println!("something else")
//                            }
//                        }
//                        println!("Error");
//                        println!("{}", e);
//                        tx.send(())
//                    }
//                }
//            }),
//    );
//
//    match rx.wait() {
//        Ok(_) => {
//            println!("OK!")
//        }
//        Err(e) => {
//            println!("{}", e)
//        }
//    }
//    // rx.and_then(|r| {
//    //     println!("ending...");
//    //     Ok(())
//    // }).wait().unwrap();
//
//    // rx.wait().unwrap();
//}

/// slice/d Streams enhances Redis Streams with disk persistence.

pub const DEFAULT_PACK_SIZE: u32 = 65500;                    // ~64KB
pub const DEFAULT_SEGMENT_SIZE: u32 = 1024 * 1024 * 64 - 64; // ~64MB

pub static mut WRITER_MAP_SIZE: usize = 1024 * 1024;
pub const RAX_EMPTY_MEM_USAGE: u64 = 24;

pub const COMPRESS_NONE: i32 = 0;
pub const COMPRESS_LZ4: i32 = 1;
pub const COMPRESS_ZSTD: i32 = 2;

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

pub const STREAM_FLAG_NONE: i32 = 0;               /* No special flags. */
pub const STREAM_FLAG_LZ4: i32 = (1 << 0);          /* Packs are compressed with LZ4 on disk */

pub const SEGMENT_FLAG_UNLOADED: i32 = 0;               /* No special flags. */
pub const SEGMENT_FLAG_LOADED: i32 = (1 << 0);          /* Segment is loaded */
pub const SEGMENT_FLAG_TAILER: i32 = (1 << 1);          /* Segment is available for writing */

pub const BLOCK_FLAG_UNLOADED: i32 = 0;               /* No special flags. */
pub const PACK_FLAG_LOADED: i32 = (1 << 0);          /* Segment is loaded */
pub const BLOCK_FLAG_TAILER: i32 = (1 << 1);          /* Segment is available for writing */

pub type PageID = u64;
pub type RecordID = StreamID;

/// The base of a record is a contiguous chunk of memory in listpack format,
/// starting after the ID delta.
pub struct Record {
    id: StreamID,
    field_names: RecordFieldNames,
}

pub struct RecordFieldNames {
    p: *mut u8,
    len: u32,
}

impl Record {
//    pub fn new(id: RecordID, fields: &[SDS]) -> Record {
//    }
}


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

struct StreamIOStats {
    pub faults: u64,
    pub read_ahead: u64,
    pub read_io_latency: u64,
}

struct ConsumerGroup {
    /// Each ConsumerGroup has it's own Redis Stream which acts like a view.
    stream: Rc<Stream>,
    /// Deduplication check.
    /// If key exists, but NACK is null then request is rejected.
    dupe: Option<RaxMap<&'static [u8], NACK>>,
    pending: RaxMap<StreamID, NACK>,

    /// Reserve the
    pins: Vec<Pin>,
}

pub struct Cursor {
    rev: bool,
    pins: Vec<Pin>,
}

/// Pending (not yet acknowledged) message in a consumer group.
struct NACK {
    delivery_time: u64,
    delivery_count: u64,
    consumer: std::sync::Arc<Consumer>,
    dupe: Sds,
}

struct Dupe {
    /// Keeps track of whether this key is currently being processed.
    /// If it is
    in_flight: atomic::AtomicBool,
}

struct Consumer {
    raw: *mut raw::streamConsumer,
}


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

impl std::error::Error for StreamError {
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


trait Command {

}

struct CommandAdd {
    record: Record,
}

///
struct CommandClaim {

}

/// Consumer groups utilize a message queue type pattern of requiring an ACK
/// to determine that a pending message was processed. Furthermore,
/// Request/Reply pattern is supported by the optional reply record on ACK.
struct CommandAck {
    /// An optional reply record.
    reply: Option<Record>,
}

/// Group read
struct CommandReadGroup {
    stream: Rc<Stream>,
    group: Rc<ConsumerGroup>,
    consumer: Rc<Consumer>,
    timeout: u32,
    count: u16,
}

impl CommandReadGroup {
    fn run() {

    }
}

///
struct CommandRead {
    stream: Rc<Stream>,
    from_id: StreamID,
    pin: Rc<Pin>,
    count: u16,
    timeout: u32,
    rev: bool,
}

impl CommandRead {
    fn run() {
        //
    }
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
struct StreamManager {
    /// The maximum amount of RAM that is allowed for the collective
    /// usage of all Streams and all of it's in-memory representations.
    max_memory: u64,
    mem_usage: std::sync::atomic::AtomicUsize,

//    counter: std::sync::atomic::AtomicU64,
    dir: &'static std::path::Path,
    bucket: SDS,
    streams: RaxMap<SDS, Rc<Stream>>,
    data: spin::Mutex<RaxMap<u64, Rc<Listpack>>>,
    pins: RaxMap<u64, Pin>,
}

impl StreamManager {
    pub fn new(bucket: SDS, path: &'static std::path::Path) -> StreamManager {
        StreamManager {
            max_memory: 0,
            mem_usage: std::sync::atomic::AtomicUsize::new(0),
//            counter: std::sync::atomic::AtomicU64::new(),
            dir: path.clone(),
            bucket: bucket,
            streams: RaxMap::new(),
            data: spin::Mutex::new(RaxMap::new()),
            pins: RaxMap::new(),
        }
    }

    pub fn start(&mut self) {
        std::thread::spawn(move || {

        });
    }

    pub fn create_stream(&mut self, name: SDS) -> Result<Rc<Stream>, StreamError> {
        unsafe {
//            let mut streams = &mut self.streams;
//            match streams.find(name) {
//                Some(s) => return Err(StreamError::Exists)
//            }

//            if streams.exists(name.clone()) {
//                return Err(StreamError::Exists)
//            }

            let stream = Rc::from_raw(leak_raw(Stream {
                id: 0,
                name: name,
                counter: 0,
                writer: None,
                segments: RaxMap::new(),
                config: StreamConfig {
                    max_pack_size: DEFAULT_PACK_SIZE,
                    max_segment_size: DEFAULT_SEGMENT_SIZE,
                    compression: COMPRESS_NONE,
                },
                groups: RaxMap::new(),
            }));

//            match streams.insert_ptr(name, Rc::into_raw(stream.clone()) as *const _ as *mut u8) {
//                Ok(_) => {}
//                Err(e) => {
//                    match e {
//                        RaxError::OutOfMemory() => return Err(StreamError::OutOfMemory),
//                        _ => return Err(StreamError::Generic(String::new()))
//                    }
//                }
//            }

            // Write to global AOF.
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

            // MO.X SEGMENT ADD mystream 10 start 64mb max 64mb pack 64kb
            // MO.STREAM REMOVE SEGMENT mystream 10

            // MO.XADD
            // MO.XREADGROUP
            // MO.XREAD
            // MO.XRANGE

            Ok(stream.clone())
        }
    }

    fn write(&mut self, stream: Rc<Stream>, id: &StreamID, record: &Record) {

    }

    fn add_segment(&mut self, mut stream: Rc<Stream>) {
        let mut s = Rc::get_mut(&mut stream).unwrap();

        let min_key = match s.last_id() {
            Some(id) => next_id(&id),
            None => StreamID::default()
        };

        unsafe {
            let segments = &mut s.segments;

            let segment = Rc::from_raw(leak_raw(Segment {
//                id: self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                inner: Some(SegmentIndex {
                    mmap: None,
                    packs: RaxMap::new(),
                }),
            }));

            // Insert into segment Rax.
            match segments.insert_ptr(min_key, Rc::into_raw(segment.clone()) as *mut u8) {
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

    pub fn drain_queue(&self) {

    }
}

struct Stream {
    id: u64,
    name: SDS,
    counter: u64,
    /// Each stream has a single writer which has the tail segment.
    writer: Option<SegmentWriter>,
    /// Segments
    segments: RaxMap<StreamID, Rc<Segment>>,
    config: StreamConfig,
    groups: RaxMap<&'static [u8], ConsumerGroup>,
}

impl Drop for Stream {
    fn drop(&mut self) {
        sliced::alloc::free(self);
        println!("dropped Stream");
    }
}


/// Future when the next writer becomes available.
struct WriterFuture {
    pub writer: Option<Rc<SegmentWriter>>,
}

/// Future of when Pin's data is faulted in.
struct CreateFuture<'a> {
    pub path: &'a std::path::Path,
    pub size: u32,
    pub mmap: Option<MmapMut>,
    pub err: Option<StreamError>,
}

struct TruncateFuture<'a> {
    pub path: &'a std::path::Path,
    pub size: u32,
    pub mmap: MmapMut,
    pub err: Option<StreamError>,
}

/// Future when segment index is faulted in.
struct SegmentFuture {
    pub segment: Option<Rc<Segment>>,
}

/// Future of when Pin's data is faulted in.
struct PinFuture {
    pub pin: Option<Rc<Pin>>,
    pub err: Option<StreamError>,
}

struct ReadFuture {
    segment: Rc<SegmentIndex>,
    future: Arc<Box<ReadTask>>,
}
struct ReadTask {
    mmap: Mmap,
    offset: u32,
    length: u32,
    count: u16,
    data: Option<Listpack>,
    err: Option<StreamError>,
//    callbacks: Vec<Fn(Box<ReadFuture>)>,
}

struct SyncFuture {
    mmap: MmapMut,
}

type SegmentID = StreamID;


struct SegmentLoadTask {
    callbacks: Vec<usize>,
}

enum SegmentTask {
    /// Create a new segment file with specified sequence number.
    Create(Arc<Box<CreateFuture<'static>>>),

    Read(Arc<Box<ReadTask>>),

    Sync(Arc<Box<SyncFuture>>),

    /// Shutdown the store and close up all file handles and flush
    /// all pending data to disk.
    Shutdown,
}

/// Manages file operations on segment files.
struct SegmentStore {
    loads: RaxMap<SegmentID, SegmentIndex>,
    tx: std::sync::mpsc::Sender<SegmentTask>,
    thread: std::thread::JoinHandle<i32>,

    completed: Arc<spin::Mutex<std::collections::VecDeque<SegmentTask>>>,
}

impl SegmentStore {
    pub fn start(path: &std::path::Path) -> Result<SegmentStore, StreamError> {
        let (tx, rx) = std::sync::mpsc::channel();
        let completed = Arc::new(spin::Mutex::new(std::collections::VecDeque::with_capacity(1024)));
        let c2 = completed.clone();

        let handle = std::thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(task) => {
                        match task {
                            SegmentTask::Create(ref create) => {

                            }
                            SegmentTask::Read(ref read) => {

                            }
                            SegmentTask::Sync(ref sync) => {}
                            SegmentTask::Shutdown => {}
                        }

                        let mut lock = c2.lock();
                        lock.push_back(task);
                        drop(lock);
                    }
                    Err(e) => {}
                }
            }
            0
        });
        Ok(SegmentStore {
            loads: RaxMap::new(),
            tx: tx.clone(),
            thread: handle,
            completed: completed.clone(),
        })
    }

    pub fn process_completed(&self) {
        let mut lock = self.completed.lock();
        let mut count = 0;
        while let Some(task) = lock.pop_front() {
            match task {
                SegmentTask::Create(ref create) => {

                }
                SegmentTask::Read(ref read) => {

                }
                SegmentTask::Sync(ref sync) => {

                }
                SegmentTask::Shutdown => {

                }
            }

            count = count + 1;
            if count == 1024 {
                break
            }
        }
        drop(lock);
    }
}

pub const BLOB_LOCATION_FS: u8 = 1;
pub const BLOB_LOCATION_OBJECT: u8 = 2;

/// Segments contain a sequence of Packs.
struct Segment {
    inner: Option<SegmentIndex>,
}

impl Segment {

}

impl Drop for Segment {
    fn drop(&mut self) {
        sliced::alloc::free(self);
        println!("dropped Segment");
    }
}

struct SegmentIndex {
    /// mmap file handle
    mmap: Option<Mmap>,
    /// index of Packs. Each Pack may be faulted in and out based
    /// on demand.
    packs: RaxMap<RecordID, Rc<Pack>>,

}

/// Packs represent a Listpack of records in standard Redis Streams format.
/// Packs can be pinned in memory to guarantee faults will not occur. This
/// is particulary important for Consumer Groups since it does not copy the
/// data for it's NACK struct in it's pending entries list (pel).
pub struct Pack {
    /// Internal ID to use for disk I/O.
    id: u64,
    /// Index of Pack within segment's Pack index
//    seq: u16,
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
    data: Option<sliced::alloc::rc::Weak<Pin>>,
}

impl Drop for Pack {
    fn drop(&mut self) {
        self.data = None;
        sliced::alloc::free(self);
        println!("dropped Pack");
    }
}

trait PackData {

}

/// Leases and pins a pack of data into memory as well as
/// it's parent path to the stream to ensure everything stays
/// intact.
///
/// This is the essence of Streams memory management. Pins allow
/// stream operations to happen at main memory speeds with the caveat
/// that a pinned listpack may incur an I/O fault. This changes the
/// behavior into a blocking operation with a pinned future.
pub struct Pin {
    segment: Rc<Segment>,
    pack: Rc<Pack>,
    data: spin::Mutex<Option<Listpack>>,
}

impl Drop for Pin {
    fn drop(&mut self) {
//        let mut lock = self.data.lock();
//        *lock.as_mut() = None;
//        drop(lock);
        sliced::alloc::free(self);
        println!("dropped Pin");
    }
}




/// Writes happen on the event-loop by writing to an mmap'ed region
/// that is or should be locked into memory. If the write cannot happen
/// without a page fault, then it turns into a Redis blocking write which
/// is non-blocking from the event-loop by blocking to the client connection.
/// The goal is to minimize that happening.
/// All writes are ordered.
struct SegmentWriter {
    last_id: StreamID,
    segment: Rc<Segment>,
    /// Tail can just use a Pin
    tail: Rc<Pin>,
    aof: Arc<AOF>,
    next_aof: Arc<AOF>,
}

impl SegmentWriter {
    pub fn write(&mut self) {

    }
}

const AOF_GROW_1MB: u64 = 1024 * 1024;
const AOF_GROW_2MB: u64 = 1024 * 1024 * 2;
const AOF_GROW_4MB: u64 = 1024 * 1024 * 4;
const AOF_GROW_8MB: u64 = 1024 * 1024 * 8;
const AOF_GROW_16MB: u64 = 1024 * 1024 * 16;
const AOF_GROW_32MB: u64 = 1024 * 1024 * 32;
const AOF_GROW_64MB: u64 = 1024 * 1024 * 64;

/// Memory mapped Append-only file.
struct AOF {
    inner: spin::Mutex<AOFInner>
}

impl AOF {
    pub fn new(f: File, size: u64) -> IoResult<AOF> {
        let len = f.metadata().unwrap().len();

        // Truncate
        match len {
            0 => {
                f.set_len(size);
            }
            _ => {
                // Let's not allow shrinking here.
                if len > size {
                    return Err(IoError::from(std::io::ErrorKind::UnexpectedEof))
                }
                match f.set_len(size) {
                    Err(e) => return Err(e),
                    _ => {}
                }
            }
        }

        match unsafe { sliced::mmap::MmapOptions::new().map_mut(&f) } {
            Ok(map) => Ok(AOF {
                inner: spin::Mutex::new(AOFInner {
                    file: f,
                    mmap: map,
                    offset: len,
                    length: len,
                    size: len,
                })
            }),
            Err(e) => Err(e)
        }
    }

    pub fn try_read(&self, offset: u64, buf: *mut u8, size: usize) -> IoResult<()> {

        Ok(())
    }

    ///
    pub fn try_append(&self, buf: *mut u8, size: usize) -> IoResult<bool> {
        let mut lock = self.inner.lock();

        let mmap = lock.mmap.as_mut_ptr();
        if lock.length + (size as u64) > lock.size + 1 {
            drop(lock);
            // Need to grow the file or use the next one.
            return Ok(false)
        }

        unsafe {
            // memcpy
            std::ptr::copy_nonoverlapping(
                buf as *const u8,
                mmap.offset(lock.length as isize),
                size
            );
            // Move the EOF byte to the new end.
            *mmap.offset(lock.length as isize + 1) = sliced::redis::listpack::EOF;
        }

        // Do not include the EOF byte so it will be overwritten.
        lock.length = lock.length + (size as u64);

        drop(lock);
        Ok(true)
    }

    ///
    pub fn truncate(&self, size: u64) -> IoResult<()> {
        let mut lock = self.inner.lock();

        // Truncate the file.
        (&mut lock.file).set_len(size);

        // fsync existing contents.
        lock.mmap.flush();

        // mmap the whole file.
        match unsafe { sliced::mmap::MmapOptions::new()
            .offset(0)
            .len(size as usize)
            .map_mut(&lock.file)
        } {
            Ok(map) => {
                lock.mmap = map;
                drop(lock);
                return Ok(())
            }
            Err(e) => {
                drop(lock);
                return Err(e)
            }
        }
    }
}

struct AOFInner {
    file: File,
    mmap: MmapMut,
    /// The logical tail of the file is mlock'ed into memory to allow
    /// writes directly from an event-loop.
    offset: u64,
    length: u64,
    size: u64,
}


impl Pack {
}


impl Stream {
    #[inline]
    pub fn last_id(&self) -> Option<StreamID> {
        self.segments.last_key()
    }
}


#[derive(Copy)]
#[repr(C)]
pub struct StreamID {
    pub ms: u64,
    pub seq: u64,
}

impl std::fmt::Display for StreamID {
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
    fn partial_cmp(&self, other: &StreamID) -> Option<std::cmp::Ordering> {
        if self.ms > other.ms {
            Some(std::cmp::Ordering::Greater)
        } else if self.ms < other.ms {
            Some(std::cmp::Ordering::Less)
        } else if self.seq > other.seq {
            Some(std::cmp::Ordering::Greater)
        } else if self.seq < other.seq {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Equal)
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
    time::precise_time_ns() / 1_000_000
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


pub mod raw {
    use ::StreamID;
    use sliced::redis::listpack::*;
    use sliced::alloc::*;
    use sliced::redis::rax::*;
    use sliced::redis::sds::*;
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
}