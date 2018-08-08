use alloc::ALLOCATOR as A;
use redis::api;
use redis::listpack::{Listpack, Value};
use smallvec::SmallVec;
use super::*;
use super::id::*;

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


pub struct MasterRecord {
    fields: usize,
    len: u16,
}

pub type RecordPtr = *mut u8;


pub struct Record<'a>(&'a StreamID, &'a [Value]);

impl<'a> Record<'a> {
    pub fn validate(&self) -> Option<StreamError> {
        if self.1.len() != 0 && self.1.len() % 2 != 0 {
            return Some(StreamError::BadInput);
        }
        None
    }
}

/// slice/d Consumer Groups have some standardized fields to control
/// it's overall behavior. A naming convention is utilized over the
/// standard Redis Streams listpack format.
pub struct CGRecord {
    deadline: Option<u64>,
}

impl CGRecord {
//    pub fn parse(record: &Record) {
//        if record.1.len() < 2 || record.1.len() % 2 != 0 {
//            return Err(StreamError::BadInput);
//        }
//
//        let mut i = 0;
//        while let Some(key) = record.1[i] {
//            if let Some(value) = record.1[i + 1] {} else {
//                return Err(StreamError::BadInput);
//            }
//        }
//    }
}

/// Specialized listpack data structure. Just like a listpack
/// except it doesn't have a header in the allocation and mostly
/// append-only.
pub struct PackData {
    /// Master Stream ID.
    /// All record IDs within listpack are delta encoded from the master
    /// except for the first record in which case it "is" the ID.
    master_id: StreamID,
    /// Raw allocation.
    lp: Listpack,
//    num_fields: u16,
//    fields: *mut u8,
//    /// Pointer to the first record.
//    first: *mut u8,
}

impl Drop for PackData {
    fn drop(&mut self) {
        free(&mut self.lp)
    }
}

impl PackData {
    /// Open and setup a pack from a raw allocation without listpack header.
//    pub fn from_disk(lp: *mut u8, length: u32, count: u16) -> Option<PackData> {
//        let mut ele = lp;
//        if let Some(count) = listpack::get_i16(lp) {
//            if let Some(ele) = listpack::next(lp, ele) {
//                if let Some(deleted) = listpack::get_i16(ele) {
//                    // The on-disk
//
//                    None
//                } else {
//                    None
//                }
//            } else {
//                None
//            }
//        } else {
//            None
//        }
//    }

    pub fn new(min_alloc: u32, first: &Record) -> Result<PackData, StreamError> {
        let raw_lp = alloc(min_alloc as usize);
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
         * +----------+--------+----------+----------+---------+----------+
         * |   (ms)   |  (seq) |  offset  |  length  |  count  |    EOF   |
         * +----------+--------+----------+--=-------+---------+----------+
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
//
//
//        // append master fields.
//
//        lp = append_int(A, lp, 0).unwrap(); // count = 1

        Err(StreamError::OutOfMemory)
    }

    pub fn append(&mut self, record: &[(Value, Value)]) {
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

        // This part is identical to on-disk format.
    }

//    pub fn last(&self) -> Option<*mut u8> {
//        unsafe {
//            listpack::prev_no_hdr(self.lp, self.lp.offset(self.size as isize))
//        }
//    }

    pub fn new_from_sds(min_alloc: u32, first: &[SDS]) -> Option<PackData> {
        None
    }

    pub fn append_sds(&mut self, fields: &[SDS]) {}
}

/// Aggressively cache the Pack
pub struct StrongPackCursor {
    last_accessed: u64,
    /// Strongly reference the pack. This will enforce
    /// that the Pack remain in memory.
    pack: Rc<Pack>,
    /// Points to an element when iterating and reverts
    /// back to an offset within listpack when it's not.
    /// Since the listpack allocation may change across
    /// command calls we must revert to the offset.
    ele: usize,
}

impl StrongPackCursor {
    /// Downgrades to a WeakPackCursor. During memory pressure,
    /// we can try to free a Pack by downgrading all strong references
    /// to weak references.
    pub fn downgrade(&mut self) -> WeakPackCursor {
        WeakPackCursor {
            last_accessed: self.last_accessed,
            pack: Rc::downgrade(&mut self.pack),
            ele: self.ele,
        }
    }
}

/// Weakly cache the Pack.
pub struct WeakPackCursor {
    last_accessed: u64,
    /// Weakly reference the pack. On resume, we must check
    /// if the pack is still in memory.
    pack: Weak<Pack>,
    /// Points to an element when iterating and reverts
    /// back to an offset within listpack when it's not.
    /// Since the listpack allocation may change across
    /// command calls we must revert to the offset.
    ele: usize,
}

impl Drop for WeakPackCursor {
    fn drop(&mut self) {}
}

impl WeakPackCursor {}