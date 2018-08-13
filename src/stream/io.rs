use spin::Mutex;
use std::cell::Cell;
use std::fs;
use std::marker;
use std::mem;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Weak as ArcWeak};
use std::sync::mpsc;
use std::thread;

use crate::alloc::{alloc, free};
use crate::mmap::{Mmap, MmapMut, MmapOptions};
use crate::redis::listpack::Listpack;
use crate::redis::rax::RaxMap;
use crate::redis::sds::SDS;

use super::*;
//use super::id::{next_id, StreamID};


pub const BLOB_LOCATION_FS: u8 = 1;
pub const BLOB_LOCATION_OBJECT: u8 = 2;

type SegmentID = StreamID;

/// Future when the next writer becomes available.
struct WriterFuture {
//    pub writer: Option<Rc<SegmentWriter>>,
}

/// Future of when Pin's data is faulted in.
struct CreateFuture<'a> {
    pub path: &'a Path,
    pub size: u32,
    pub mmap: Option<MmapMut>,
    pub err: Option<StreamError>,
}

struct TruncateFuture<'a> {
    pub path: &'a Path,
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
    segment: Rc<Segment>,
    future: Arc<Box<ReadTask>>,
}

/// Reads a contiguous block of memory from a file.
struct ReadTask {
    mmap: Arc<Mutex<Mmap>>,
    offset: u32,
    length: u32,
    count: u16,
    data: usize,
    err: Option<StreamError>,
//    callbacks: Vec<Fn(Box<ReadFuture>)>,
}

struct SyncFuture {
    mmap: MmapMut,
}

struct SegmentLoadTask {
    callbacks: Vec<usize>,
}

struct OpenSegment {
    id: StreamID,
}

enum Task {
    /// Create a new segment file with specified sequence number.
    Create(Arc<Box<CreateFuture<'static>>>),

    Read(Arc<Box<ReadTask>>),

    Sync(Arc<Box<SyncFuture>>),

    /// Shutdown the store and close up all file handles and flush
    /// all pending data to disk.
    Shutdown,
}

pub enum StorageType {
    File,
    Object,
}

pub struct IoFuture<T: ?Sized + Send + 'static> {
    marker: marker::PhantomData<T>,
}

enum Location {
    Local,
    Remote,
}

struct SegmentEntry {
    location: Cell<Location>,
}

fn base_mem_usage(s: &StorageService) {}

/// Manages persistence of Streams through segment files.
///
/// > <root>
/// >> {redis db num}
/// >>> {redis db num}/{streamID}
///
/// Stream Layout
/// -- 0.dat (tail segment "writer" file)
/// -- n.dat (next segment pre-allocated)
/// -- dict.txt = AOF index of segments
/// -- {StreamID}.dat = immutable segment file
/// -- {StreamID}.dat
pub struct StorageService {
    dir: String,
    /// Background thread sender.
    bg_sender: mpsc::SyncSender<Task>,
    /// Background thread handle.
    bg_thread: thread::JoinHandle<i32>,
    ev_sender: mpsc::SyncSender<Task>,
    ev_receiver: mpsc::Receiver<Task>,

    /// A map of futures. Each future may hold many continuations.
    /// Each continuation is likely a client based command that all
    /// require the same Pack or AOF to complete. We handle the race
    /// through the future.
    futures: RaxMap<u64, IoFuture<Task>>,

    /// Keep track of memory used by all tasks, futures and continuations.
    mem_usage: usize,
    /// Amount of disk space mmap'ed into virtual memory.
    mmap_total: usize,

    /// The size of the local file-system
    disk_size: usize,
    /// The number of bytes the local file-system reports as being available.
    disk_avail: usize,

    /// The amount of disk spaced that is pinned likely due to existing writers
    /// and readers. The system will aggressively unpin segments when it's able
    /// to do so and then rely on the cache management to keep what it can around
    /// based on demand.
    disk_cache_pinned: usize,
    /// The amount of disk usage that is desired to always be used. The system will
    /// not evict archived segments until this number is reached or some other eviction
    /// indicator is tripped like "last-used".
    disk_cache_min: usize,
    /// Once this threshold is exceeded
    disk_cache_max: usize,

    /// This parameter allows the "disk_cache_min" rule to be broken for segments that
    /// haven't been needed since "evict_stale_age" ago. This can usually be set to
    /// a couple of days or even a week. With streams it's relatively rare to request
    /// something old and when it is requested it's likely a single processor going
    /// through the whole stream one time.
    evict_stale_age: u64,
}

impl StorageService {
    pub fn start(path: &Path) -> Result<StorageService, StreamError> {
        // Background channel. Use a sync channel to handle back-pressure.
        let (
            bg_sender,
            bg_receiver
        ) = mpsc::sync_channel(super::max_io_backlog());

        // Event-loop channel. Use a sync channel to handle back-pressure.
        // The event-loop will never block and if sending a task on the background channel
        // will block, it will fail fast and the error will be pushed back to the client.
        let (
            ev_sender,
            ev_receiver
        ) = mpsc::sync_channel(super::max_io_backlog());

        // Spawn background thread.
        let handle = thread::spawn(move || {
            loop {
                match bg_receiver.recv() {
                    Ok(task) => {
                        match task {
                            // Create and open a new file.
                            Task::Create(ref create) => {

                            }
                            Task::Read(ref read) => {}
                            Task::Sync(ref sync) => {}
                            Task::Shutdown => {}
                        }
                    }
                    Err(e) => {}
                }
            }
            0
        });

        // Do we need to create the directory?
        if !path.exists() {
            match fs::create_dir_all(path) {
                Ok(_) => {}
                Err(_) => {
                    return Err(StreamError::CreateDir(
                        String::from(path.to_str().unwrap_or("<empty>"))
                    ))
                }
            }
        } else if !path.is_dir() {
            // The path exists, however it's not a directory.
            return Err(StreamError::NotDir(
                String::from(path.to_str().unwrap_or("<empty>"))
            ))
        } else {
            // Do recovery
            match path.read_dir() {
                Ok(_) => {
                    // Find
                }
                Err(_) => return Err(StreamError::ReadDir(
                    String::from(path.to_str().unwrap_or("<empty>"))
                ))
            }
        }

        Ok(StorageService {
            dir: String::from(path.to_str().unwrap_or("<unknown>")),
            bg_sender: bg_sender.clone(),
            bg_thread: handle,
            ev_sender: ev_sender.clone(),
            ev_receiver,
            futures: RaxMap::new(),

            mem_usage: mem::size_of::<StorageService>(),

            mmap_total: 0,

            /// Size of local file-system.
            disk_size: 0,
            disk_cache_pinned: 0,

            /// Bytes available on local file-system.
            disk_avail: 0,

            ///
            disk_cache_min: 0,
            disk_cache_max: 0,

            ///
            evict_stale_age: 0,

        })
    }

    /// Must be called from the event-loop. This function polls the completed
    /// background work and invokes the continuation for each task. It will invoke
    /// the configured max so we don't cause too much lag on the event-loop.
    pub fn poll(&self) {
//        let mut lock = self.completed.lock();
//        let mut count = 0;
//        while let Some(task) = lock.pop_front() {
//            match task {
//                Task::Create(ref create) => {}
//                Task::Read(ref read) => {}
//                Task::Sync(ref sync) => {}
//                Task::Shutdown => {}
//            }
//
//            count = count + 1;
//            if count == 1024 {
//                break;
//            }
//        }
//        drop(lock);
    }
}

impl Drop for StorageService {
    fn drop(&mut self) {
    }
}

/// After a segment becomes immutable it is immediately scheduled to be archived
/// into archive storage which is either another mounted file system or an object
/// storage system like Amazon S3. Once, a file is confirmed to be archived, then
/// it will be un-pinned from the local file-system and available for removal when
/// the system would like to free up some disk space.
pub struct ArchiveService {

}

#[cfg(test)]
mod tests {
    #[test]
    fn start_storage_service() {
        println!("starting StorageService...");
        let mut x = 0;
        x += 1;
        println!("{}", x);

        println!("stopping StorageService...");
    }
}