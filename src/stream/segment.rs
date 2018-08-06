use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::collections::VecDeque;

use spin::Mutex;

use ::alloc::{free, alloc};
use ::alloc::arc::{Arc, Weak as ArcWeak};
use ::alloc::rc::{Rc, Weak};
use ::alloc::raw_vec::RawVec;

use ::mmap::{Mmap, MmapMut, MmapOptions};
use ::redis::listpack::Listpack;
use ::redis::rax::RaxMap;
use ::redis::sds::SDS;



use super::*;
use super::id::{StreamID, next_id};

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

pub enum SegmentMedia {
    File,
    Object,
}

/// Manages file operations on segment files.
struct SegmentStore {
    tx: mpsc::Sender<SegmentTask>,
    thread: thread::JoinHandle<i32>,
    completed: Arc<Mutex<VecDeque<SegmentTask>>>,
}

impl SegmentStore {
    pub fn start(path: &Path) -> Result<SegmentStore, StreamError> {
        let (tx, rx) = mpsc::channel();
        let completed = Arc::new(Mutex::new(VecDeque::with_capacity(1024)));
        let c2 = completed.clone();

        let handle = thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(task) => {
                        match task {
                            // Create and open a new file.
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

