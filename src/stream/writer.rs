use crate::redis::listpack;
use crate::redis::listpack::{MemoizedValue, UnsafeAppender, Value};
use spin::Mutex;
use std::ptr;
use std::sync::Arc;
use super::*;
use super::record::*;

// Redis Streams entry flags
pub const STREAM_ITEM_FLAG_NONE: i32 = 0;               /* No special flags. */
pub const STREAM_ITEM_FLAG_DELETED: i32 = (1 << 0);     /* Entry is delted. Skip it. */
pub const STREAM_ITEM_FLAG_SAMEFIELDS: i32 = (1 << 1);  /* Same fields as master entry. */
pub const STREAM_ITEM_FLAG_SLOT: i32 = (1 << 2);        /* Has slot number */
pub const STREAM_ITEM_FLAG_TX: i32 = (1 << 3);          /* Has tx key */
pub const STREAM_ITEM_FLAG_DEDUPE: i32 = (1 << 4);      /* Has de-duplication key */

/// Reserved field name for Slot number chosen.
pub const FIELD_SLOT: &'static [u8] = b"[";
pub const FIELD_TX_KEY: &'static str = "^";
pub const FIELD_CALLER_ID: &'static str = "#";
pub const FIELD_REPLY_MAILBOX: &'static str = "@";
pub const FIELD_DUPE_KEY: &'static str = "?";
pub const FIELD_DEFER: &'static str = "!";

pub struct StreamStats {
    total_records: u64,
    total_packs: u64,
    total_segments: u64,
    total_same_fields: u64,
    total_deleted: u64,
    avg_record_size: f32,
    avg_fields_per_record: f32,
    avg_records_per_pack: f32,
}

pub struct StreamArchiveStats {
    segments: u64,
    total_size: u64,
    total_size_compressed: u64,
}

struct SegmentReader {
    /// When None, file needs to be located and opened on BG thread.
    file: Arc<Mutex<Option<mmap::Mmap>>>,
    /// Pack index
    packs: map::RcRax<StreamID, Pack>,
}

struct SegmentWriter {
    /// Active AOF.
    /// Path = {root_dir}/stream_id/0.dat
    /// Protected by a spin Mutex since it is shared with an I/O thread.
    aof: Option<Arc<Mutex<aof::AOF>>>,
    /// Last used StreamID. The next ID must be greater than the previous.
    last_id: StreamID,
    /// Master ID of the tail pack.
    /// All record IDs within listpack are delta encoded from the master
    /// except for the first record in which case it "is" the ID.
    tail_master_id: StreamID,
    /// The last pack of the segment. New writes go here.
    tail: Option<Rc<Pack>>,
    /// Number of master fields.
    tail_num_fields: u16,
    /// Pointer to the first field if "tail_num_fields" > 0 else null_mut()
    tail_fields: listpack::element,
    /// Size of tail Pack's memory allocation. The StreamWriter will take
    /// care of reallocating the tail as necessary and according to the
    /// configuration.
    tail_alloc: u32,

}

struct SegmentContinuation {

}

struct SegmentFutures {
    read: RaxMap<StreamID, SegmentContinuation>
}

///
struct SegmentModel {
    handle: SegmentHandle,
    ///
    packs: map::RcRax<StreamID, Pack>,
    futures: RaxMap<u64, u64>,
}

impl SegmentModel {
    pub fn new() {

    }
}

enum SegmentHandle {
    /// File should be on local file-system, but it is not currently open.
    Local,

    /// Currently waiting on background to finish opening.
    Opening(Vec<u64>),

    ///
    Immutable(Arc<Mutex<mmap::Mmap>>),

    /// File is opened
    Mutable(Arc<Mutex<mmap::MmapMut>>),

    /// File should be on remote file-system only and needs to
    /// be downloaded and upgraded to local to access it.
    Archived,

    /// File is currently in the process of moving from
    /// archive storage to local file-system.
    Downloading(u64, u64),

    ///
    Uploading(Arc<Mutex<mmap::Mmap>>),

    /// File should be on remote file-system only and needs to
    /// be downloaded and upgraded to local to access it.
    LocalAndArchived,

    ///
    Error(String),
}

/// Mutations to a stream is managed by the StreamWriter.
pub struct StreamWriter {
    stream: Rc<Stream>,

    /// ID of the segment is the min StreamID available within it.
    segment_id: StreamID,
    /// The current segment index.
    segment: Rc<Segment>,
    /// Active AOF.
    /// Path = {root_dir}/stream_id/0.dat
    /// Protected by a spin Mutex since it is shared with an I/O thread.
    aof: Option<Arc<Mutex<aof::AOF>>>,

    /// Last used StreamID. The next ID must be greater than the previous.
    last_id: StreamID,
    /// Master ID of the tail pack.
    /// All record IDs within listpack are delta encoded from the master
    /// except for the first record in which case it "is" the ID.
    tail_master_id: StreamID,
    /// The last pack of the segment. New writes go here.
    tail: Option<Rc<Pack>>,
    /// Number of master fields.
    tail_num_fields: u16,
    /// Pointer to the first field if "tail_num_fields" > 0 else null_mut()
    tail_fields: listpack::element,
    /// Size of tail Pack's memory allocation. The StreamWriter will take
    /// care of reallocating the tail as necessary and according to the
    /// configuration.
    tail_alloc: u32,

    /// Next segment that is prepared.
    next_segment: Rc<Segment>,
    /// Next AOF for the next segment that is prepared.
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
    ///
    /// Note: A pack MUST have at least 1 record and if that record is
    /// larger than this number then it breaks this guideline. Otherwise,
    /// this will not be exceeded.
    pack_max: u32,

    /// Currently waiting to here back about the grow request.
    growing: bool,
}

struct NeedMoreDiskSpace {}

#[inline]
///
pub fn realloc_for_write(p: *mut u8, size: usize) -> *mut u8 {
    realloc(p, size)
}

impl StreamWriter {
    pub fn next_id(&mut self) -> StreamID {
        self.last_id = id::next_id(&self.last_id);
        self.last_id
    }

    /// Attempts to
    pub fn try_write(&mut self, kv: &mut [listpack::Value]) -> Result<(), StreamError> {
        match self.aof {
            Some(ref aof) => {
                match aof.try_lock() {
                    Some(ref mut locked) => {
                        // We have exclusive access to the AOF.
                        let segment_length = locked.len();
                        let offset = locked.offset();

                        // Get the tail pack.
                        match self.tail {
                            Some(ref mut tail) => {}
                            None => {
                                // Determine new
                                let mut pack = Pack::new();
//                                pack.data = Some(record::PackData::new());
                                self.tail = Some(Rc::new(pack));
                            }
                        };

                        // Get listpack.
//                        let mut data = match tail.data {
//                            Some(mut lp) => lp,
//                            None => Listpack::new(),
//                        };
//
//                        let marker = data.bytes();

                        // Create record ID.
//                        let id = self.next_id();

                        // Write it to the file.
//                        locked.try_append(ptr::null_mut(), 0);


//                        drop(locked);

                        Ok(())
                    }
                    // Background thread has the lock.
                    // It's the commands responsibility to determine if it wants
                    // to create a Future and wait for the availability of the AOF.
                    None => return Err(StreamError::WouldBlock)
                }
            }
            None => return Err(StreamError::WouldBlock)
        }
    }

    fn new_pack(&mut self, id: &StreamID, kv: &mut [Value]) -> Result<PackData, StreamError> {
        let raw_lp = alloc(self.pack_min as usize);
        if raw_lp.is_null() {
            return Err(StreamError::OutOfMemory);
        }

        let mut lp = Listpack::from_raw(raw_lp);

        /*
         * The master entry "in-memory" layout is composed like in the following example:
         *
         * +-------+---------+------------+---------+--/--+---------+---------+-+
         * | count | deleted | num-fields | field_1 | field_2 | ... | field_N |0|
         * +-------+---------+------------+---------+--/--+---------+---------+-+
         *
         * The "on-disk" layout is a bit different to ensure append-only writes.
         * Header (bytes, items), Count, Deleted are not in the on-disk representation
         * at the same location. Instead, it's encoded to the end of the listpack
         * between 2 EOF bytes.
         *
         * +----------+--------+----------+----------+---------+----------+
         * | LP-count |   EOF  | ID (ms)  | ID (seq) |   EOF   | LP-first |
         * +----------+--------+----------+--=-------+---------+----------+
         *
         * Between Packs and Index sections
         * +--------+--------+
         * |   EOF  |   IDX  |
         * +--------+--------+
         *
         * Index Entry - After all Packs at the end of the file
         * +----+-----+----------+----------+---------+----------+
         * | ms | seq |  offset  |  length  |  count  |    EOF   |
         * +----+-----+----------+--=-------+---------+----------+
         */
        // count = 1
        if !lp.append(1) {
            return Err(StreamError::OutOfMemory);
        }
        // deleted = 0
        if !lp.append(0) {
            return Err(StreamError::OutOfMemory);
        }

//        lp = append_int(A, lp, 1).unwrap_or_else(|| return Err(StreamError::OutOfMemory)); // count = 1
//        lp = append_int(A, lp, 0).unwrap(); // deleted = 0
//        lp = append_int(A, lp, first.len()).unwrap(); // num_fields = 0

//        // append master fields.
//
//        lp = append_int(A, lp, 0).unwrap(); // count = 1

        Err(StreamError::OutOfMemory)
    }

    fn begin_pack(
        &mut self,
        id_ms: &MemoizedValue,
        id_seq: &MemoizedValue,
        kv: &mut [MemoizedValue],
    ) {}

    /// Adds a new record only if it fits within the max_size.
    fn append(
        &mut self,
        id: &StreamID,
        kv: &mut [MemoizedValue],
        pack: &mut Pack,
    ) -> Result<(), StreamError> {
        /* Populate the listpack with the new entry. We use the following
         * encoding:
         *
         * +-----+--------+----------+-------+-------+-/-+-------+-------+--------+
         * |flags|entry-id|num-fields|field-1|value-1|...|field-N|value-N|lp-count|
         * +-----+--------+----------+-------+-------+-/-+-------+-------+--------+
         *
         * However if the SAMEFIELD flag is set, we have just to populate
         * the entry with the values, so it becomes:
         *
         * +-----+--------+-------+-/-+-------+--------+
         * |flags|entry-id|value-1|...|value-N|lp-count|
         * +-----+--------+-------+-/-+-------+--------+
         *
         * The entry-id field is actually two separated fields: the ms
         * and seq difference compared to the master entry.
         *
         * The lp-count field is a number that states the number of listpack pieces
         * that compose the entry, so that it's possible to travel the entry
         * in reverse order: we can just start from the end of the listpack, read
         * the entry, and jump back N times to seek the "flags" field to read
         * the stream full entry. */

        // Size check.
        // We calculate the total encoded size to determine overflow of the pack
        // and/or segment.
        // NOTE that this includes field names regardless of SAMEFIELDS.
        // We determine whether the existing Pack can handle the write
        // and if the AOF can take it immediately.
        // If Not, then we need to handle this as a blocking command.

        // Create StreamID diff values.
        let id_ms = MemoizedValue::new(
            Value::Int((id.ms - self.tail_master_id.ms) as i64)
        );
        let id_seq = MemoizedValue::new(
            Value::Int((id.seq - self.tail_master_id.seq) as i64)
        );

        let mut lp_size = listpack::get_total_bytes(pack.data);

        // Are there any key-values to store?
        if kv.is_empty() {
            let flag_val = MemoizedValue::new(
                Value::Int(STREAM_ITEM_FLAG_NONE as i64)
            );
            let num_fields_val = MemoizedValue::new(
                Value::Int(0)
            );
            let lp_count = MemoizedValue::new(
                Value::Int((5) as i64)
            );

            let mut encoded_size =
                flag_val.encoded_size +
                    id_ms.encoded_size +
                    id_seq.encoded_size +
                    num_fields_val.encoded_size +
                    lp_count.encoded_size;

            // Calculate new listpack size.
            let new_lp_size = encoded_size + lp_size;
            // Would it overflow?
            if new_lp_size > self.pack_max {
                //
                return Err(StreamError::Overflow);
            }
            // Maybe increase allocation?
            if self.tail_alloc < new_lp_size {
                let new_lp = realloc_for_write(
                    pack.data,
                    new_lp_size as usize,
                );
                // OOM?
                if new_lp.is_null() {
                    return Err(StreamError::OutOfMemory);
                }
                // Update tail_alloc
                self.tail_alloc = new_lp_size;
                // New allocation?
                if new_lp != pack.data {
                    pack.data = new_lp;
                }
            }

            // Do actual writes.
            unsafe {
                let mut writer = UnsafeAppender::new(
                    pack.data,
                    lp_size,
                );
                writer.append(&flag_val);
                writer.append(&id_ms);
                writer.append(&id_seq);
                writer.append(&num_fields_val);
                writer.append(&lp_count);

                listpack::set_total_bytes(pack.data, new_lp_size);
                listpack::incr_num_elements(pack.data);
            }

            Ok(())
        } else {
            // Ensure we have key-values.
            if kv.len() % 2 != 0 {
                return Err(StreamError::BadInput);
            }

            // Calculate the number of fields.
            let num_fields = kv.len() / 2;
            let mut flags = STREAM_ITEM_FLAG_NONE;

            // Do the SAMEFIELDS check.
            if self.tail_num_fields == num_fields as u16 {
                let mut ele = self.tail_fields;
                if listpack::get(ele) == kv[0].value {
                    flags |= STREAM_ITEM_FLAG_SAMEFIELDS;
                    for index in 1..num_fields {
                        match listpack::next(pack.data, ele) {
                            Some(n) => {
                                ele = n;

                                if listpack::get(ele) != kv[index * 2].value {
                                    flags = STREAM_ITEM_FLAG_NONE;
                                    break;
                                }
                            }
                            None => {
                                flags = STREAM_ITEM_FLAG_NONE;
                                break;
                            }
                        }
                    }
                }
            }

            let flag_val = MemoizedValue::new(
                Value::Int(flags as i64)
            );

            let mut encoded_size =
                flag_val.encoded_size +
                    id_ms.encoded_size +
                    id_seq.encoded_size;

            if flags == STREAM_ITEM_FLAG_SAMEFIELDS {
                let lp_count = MemoizedValue::new(
                    Value::Int((4 + num_fields) as i64)
                );
                encoded_size += lp_count.encoded_size;

                for index in 0..num_fields {
                    encoded_size += kv[index * 2 + 1].encoded_size;
                }

                // Calculate new listpack size.
                let new_lp_size = encoded_size + lp_size;
                // Would it overflow?
                if new_lp_size > self.pack_max {
                    return Err(StreamError::Overflow);
                }
                // Maybe increase allocation?
                if self.tail_alloc < new_lp_size {
                    let new_lp = realloc_for_write(
                        pack.data,
                        new_lp_size as usize,
                    );
                    // OOM?
                    if new_lp.is_null() {
                        return Err(StreamError::OutOfMemory);
                    }
                    // Update tail_alloc
                    self.tail_alloc = new_lp_size;
                    // New allocation?
                    if new_lp != pack.data {
                        pack.data = new_lp;
                    }
                }

                // Do actual writes.
                unsafe {
                    let mut writer = UnsafeAppender::new(
                        pack.data,
                        lp_size,
                    );
                    writer.append(&flag_val);
                    writer.append(&id_ms);
                    writer.append(&id_seq);

                    // Only write values since we have SAMEFIELDS flag.
                    for index in 0..num_fields {
                        writer.append(&kv[index * 2 + 1]);
                    }

                    writer.append(&lp_count);

                    listpack::set_total_bytes(pack.data, new_lp_size);
                    listpack::incr_num_elements(pack.data);
                }
            } else {
                // Store keys and values.
                let num_fields_val = MemoizedValue::new(
                    Value::Int(num_fields as i64)
                );
                let lp_count = MemoizedValue::new(
                    Value::Int((5 + kv.len()) as i64)
                );
                encoded_size += num_fields_val.encoded_size + lp_count.encoded_size;

                // Calculate encoded size of all field-values.
                for index in 0..num_fields {
                    // Add field and value encoded sizes
                    encoded_size +=
                        kv[index * 2].encoded_size +
                            kv[index * 2 + 1].encoded_size;
                }

                // Calculate new listpack size.
                let new_lp_size = encoded_size + lp_size;
                // Would it overflow?
                if new_lp_size > self.pack_max {
                    // Change to master record.

                    // Can we
                    // Can add another pack to AOF?

                    // Get next pack.
                    return Err(StreamError::Overflow);
                }
                // Maybe increase allocation?
                if self.tail_alloc < new_lp_size {
                    let new_lp = realloc_for_write(
                        pack.data,
                        new_lp_size as usize,
                    );
                    // OOM?
                    if new_lp.is_null() {
                        return Err(StreamError::OutOfMemory);
                    }
                    // Update tail_alloc
                    self.tail_alloc = new_lp_size;
                    // New allocation?
                    if new_lp != pack.data {
                        pack.data = new_lp;
                    }
                }

                // Do actual writes.
                unsafe {
                    let mut writer = UnsafeAppender::new(
                        pack.data,
                        lp_size,
                    );
                    writer.append(&flag_val);
                    writer.append(&id_ms);
                    writer.append(&id_seq);
                    writer.append(&num_fields_val);

                    for index in 0..num_fields {
                        // Add field and value encoded sizes
                        writer.append(&kv[index * 2]);
                        writer.append(&kv[index * 2 + 1]);
                    }

                    writer.append(&lp_count);

                    listpack::set_total_bytes(pack.data, new_lp_size);
                    listpack::incr_num_elements(pack.data);
                }
            }

            Err(StreamError::Overflow)
        }
    }

    pub fn finish_segment(&mut self) {
        // Write segment index to the end of it's file.

        // Rename file to the segment ID in string format "{ms}-{seq}.dat"
        // Once a file's name is changed it is guaranteed to be complete and correct.
        // If a crash happens then only the "0.dat" file in each stream needs
        // to be recovered.
    }

    /// After a crash or restart we need to figure out what the state
    /// of affairs is and fix up any issues.
    pub fn recover(&mut self) {}

    pub fn append_file(&mut self, lp_write: listpack::WriteResult) {}
}


#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn segment() {
        println!("segment");

        let mut model = SegmentModel {
            handle: SegmentHandle::Local,
            packs: map::RcRax::new(),
            futures: RaxMap::new(),
        };


    }
}