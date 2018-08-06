use std;
use std::cmp;
use std::fmt;
use std::mem;
use time;

#[derive(Copy)]
#[repr(C)]
pub struct StreamID {
    pub ms: u64,
    pub seq: u64,
}

impl StreamID {
    #[inline]
    pub fn to_big_endian(&self) -> StreamID {
        StreamID {
            ms: self.ms.to_be(),
            seq: self.seq.to_be(),
        }
    }
}

impl fmt::Display for StreamID {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
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
    fn partial_cmp(&self, other: &StreamID) -> Option<cmp::Ordering> {
        if self.ms > other.ms {
            Some(cmp::Ordering::Greater)
        } else if self.ms < other.ms {
            Some(cmp::Ordering::Less)
        } else if self.seq > other.seq {
            Some(cmp::Ordering::Greater)
        } else if self.seq < other.seq {
            Some(cmp::Ordering::Less)
        } else {
            Some(cmp::Ordering::Equal)
        }
    }
}



impl ::redis::rax::RaxKey for StreamID {
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
        (self as *const _ as *const u8, mem::size_of::<StreamID>())
    }

    #[inline]
    fn from_buf(ptr: *const u8, len: usize) -> StreamID {
        if len != mem::size_of::<StreamID>() {
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
pub fn mstime() -> u64 {
    time::precise_time_ns() / 1_000_000
}

/// Generate the next stream item ID given the previous one. If the current
/// milliseconds Unix time is greater than the previous one, just use this
/// as time part and start with sequence part of zero. Otherwise we use the
/// previous time (and never go backward) and increment the sequence.
#[inline(always)]
pub fn next_id(last: &StreamID) -> StreamID {
    let ms = mstime();
    if ms > last.ms {
        StreamID { ms, seq: 0 }
    } else {
        StreamID { ms: last.ms, seq: last.seq + 1 }
    }
}