use crate::alloc::ALLOCATOR as A;
use crate::redis::api;
use crate::redis::listpack::{Listpack, Value};
use smallvec::SmallVec;
use super::*;
//use super::id::*;

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

/// Specialized listpack data structure. Just like a listpack
/// except it doesn't have a header in the allocation and mostly
/// append-only.
pub struct PackData {
    /// A standard Listpack is used.
    lp: listpack::listpack,
}

unsafe impl Sync for PackData {}

unsafe impl Send for PackData {}

impl Drop for PackData {
    fn drop(&mut self) {
        free(&mut self.lp)
    }
}

impl PackData {
    // Open and setup a pack from a raw allocation without listpack header.
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