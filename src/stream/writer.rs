use super::*;
use std::sync::Arc;
use spin::Mutex;

/// Manages appending to a Stream's tail segment.
pub struct SegmentWriter {
    last_id: StreamID,

    /// ID of the segment is the min StreamID available within it.
    segment_id: StreamID,
    /// The current segment index.
    segment: Rc<Segment>,
    /// Active AOF.
    /// Path = {root_dir}/stream_id/0.dat
    /// Protected by a spin Mutex since it is shared with an I/O thread.
    aof: Option<Mutex<Arc<aof::AOF>>>,
    /// The last pack of the segment. New writes go here.
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

    /// Simple optimization to balance performance with memory usage.
    /// For new streams or streams that are very sparse, we can be very
    /// conservative and only allocate the minimum required. However, for
    /// streams that write lots of data it makes more sense to match the
    /// "pack_min" with the "pack_max" resulting in only a single malloc
    /// per pack.
    pack_min: u32,
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

    /// After a crash or restart we need to figure out what the state
    /// of affairs is and fix up any issues.
    pub fn recover(&mut self) {

    }

    pub fn append_file(&mut self, lp_write: listpack::WriteResult) {

    }
}