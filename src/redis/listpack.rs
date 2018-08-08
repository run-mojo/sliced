//! A listpack is encoded into a single linear chunk of memory. It has a fixed
//! length header of six bytes (instead of ten bytes of ziplist, since we no
//! longer need a pointer to the start of the last element). The header is
//! followed by the listpack elements. In theory the data structure does not need
//! any terminator, however for certain concerns, a special entry marking the
//! end of the listpack is provided, in the form of a single byte with value
//! FF (255). The main advantages of the terminator are the ability to scan the
//! listpack without holding (and comparing at each iteration) the address of
//! the end of the listpack, and to recognize easily if a listpack is well
//! formed or truncated. These advantages are, in the idea of the writer, worth
//! the additional byte needed in the representation.
//!
//!     <tot-bytes> <num-elements> <element-1> ... <element-N> <listpack-end-byte>
//!
//! The six byte header, composed of the tot-bytes and num-elements fields is
//! encoded in the following way:
//!
//! * `tot-bytes`: 32 bit unsigned integer holding the total amount of bytes
//! representing the listpack. Including the header itself and the terminator.
//! This basically is the total size of the allocation needed to hold the listpack
//! and allows to jump at the end in order to scan the listpack in reverse order,
//! from the last to the first element, when needed.
//! * `num-elements`:  16 bit unsigned integer holding the total number of elements
//! the listpack holds. However if this field is set to 65535, which is the greatest
//! unsigned integer representable in 16 bit, it means that the number of listpack
//! elements is not known, so a LIST-LENGTH operation will require to fully scan
//! the listpack. This happens when, at some point, the listpack has a number of
//! elements equal or greater than 65535. The num-elements field will be set again
//! to a lower number the first time a LIST-LENGTH operation detects the elements
//! count returned in the representable range.
//!
//! All integers in the listpack are stored in little endian format, if not
//! otherwise specified (certain special encodings are in big endian because
//! it is more natural to represent them in this way for the way the specification
//! maps to C code).

use ::alloc::*;
use std;
use std::alloc::*;
use std::mem::size_of;
use std::mem;
use std::ptr;
use std::rc;

use libc;

pub const EMPTY: &'static [u8] = &[];

/// Listpacks are composed of elements that are either an derivative of a
/// 64bit integer or a string blob. Note that the String type is pointing
/// to memory it does not own.
pub enum Value {
    ///
    Int(i64),
    ///
    String(*const u8, u32),
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match *self {
            Value::Int(v) => match *other {
                Value::Int(v2) => v == v2,
                _ => false
            },
            Value::String(p, size) => match *other {
                Value::String(p2, size2) => {
                    if size != size2 {
                        false
                    } else {
                        unsafe {
                            libc::memcmp(
                                p as *mut libc::c_void,
                                p2 as *mut libc::c_void,
                                size as libc::size_t
                            ) == 0
                        }
                    }
                }
                _ => false
            }
        }
    }
}

pub type listpack = *mut u8;
pub type element = *mut u8;

/// Used for determining how to treat the "at" element pointer during insertion.
pub enum Placement {
    /// Insert the element immediately before the specified element pointer.
    Before = 0,
    /// Insert the element immediately before the specified element pointer.
    After = 1,
}

pub struct Listpack(listpack);

unsafe impl Send for Listpack {}
unsafe impl Sync for Listpack {}

impl Listpack {
    #[inline]
    pub fn from_raw(lp: *mut u8) -> Listpack {
        Listpack(lp)
    }

    pub fn new() -> Listpack {
        unsafe { Listpack(new(ALLOCATOR)) }
    }

    #[inline(always)]
    pub fn len(&self) -> u16 {
        get_num_elements(self.0)
    }

    #[inline(always)]
    pub fn bytes(&self) -> u32 {
        get_total_bytes(self.0)
    }

    #[inline(always)]
    pub fn avg_element_size(&self) -> f32 {
        let base = self.bytes() - 7;
        if base == 0 {
            0f32
        } else {
            (base as f32) / (self.len() as f32)
        }
    }

    #[inline]
    pub fn insert<V: Into<Value>>(
        &mut self,
        v: V,
        place: Placement,
        target: element,
    ) -> Option<element> {
        unsafe {
            match insert(ALLOCATOR, self.0, v.into(), place, target) {
                Some((lp, ele)) => {
                    self.0 = lp;
                    Some(ele)
                }
                None => None
            }
        }
    }

//    #[inline]
//    pub fn insert_ref<'a, V: Into<Value>>(
//        &mut self,
//        v: &'a V,
//        place: Placement,
//        target: element,
//    ) -> Option<element> {
//        let val: Value = v.into();
//        match insert(ALLOCATOR, self.0, val, place, target) {
//            Some((lp, ele)) => {
//                self.0 = lp;
//                Some(ele)
//            }
//            None => None
//        }
//    }

    #[inline]
    pub fn insert_signed_int<T>(
        &mut self,
        v: T,
        place: Placement,
        target: element,
    ) -> Option<element>
        where
            T: Int {
        unsafe {
            match insert_signed_int(ALLOCATOR, self.0, v, place, target) {
                Some((lp, ele)) => {
                    self.0 = lp;
                    Some(ele)
                }
                None => None
            }
        }
    }

    #[inline]
    pub fn insert_string<T>(
        &mut self,
        v: T,
        place: Placement,
        target: element,
    ) -> Option<element>
        where
            T: Str {
        unsafe {
            match insert_string(ALLOCATOR, self.0, v, place, target) {
                Some((lp, ele)) => {
                    self.0 = lp;
                    Some(ele)
                }
                None => None
            }
        }
    }

    #[inline]
    pub fn replace<V: Into<Value>>(
        &mut self,
        p: element,
        v: V,
    ) -> Option<element> {
        unsafe {
            match replace(ALLOCATOR, self.0, p, v.into()) {
                Some((lp, ele)) => {
                    self.0 = lp;
                    Some(ele)
                }
                None => None
            }
        }
    }

    #[inline]
    pub fn replace_signed_int<T>(
        &mut self,
        p: element,
        v: T,
    ) -> Option<element>
        where
            T: Int {
        unsafe {
            match replace_signed_int(ALLOCATOR, self.0, p, v) {
                Some((lp, ele)) => {
                    self.0 = lp;
                    Some(ele)
                }
                None => None
            }
        }
    }

    #[inline]
    pub fn replace_string<T>(
        &mut self,
        p: element,
        v: T,
    ) -> Option<element>
        where
            T: Str {
        unsafe {
            match replace_string(ALLOCATOR, self.0, p, v) {
                Some((lp, ele)) => {
                    self.0 = lp;
                    Some(ele)
                }
                None => None
            }
        }
    }

    #[inline]
    pub fn append<V: Into<Value>>(
        &mut self,
        v: V,
    ) -> bool {
        unsafe {
            match append(ALLOCATOR, self.0, v.into()) {
                Some(lp) => {
                    self.0 = lp;
                    true
                }
                None => false
            }
        }
    }

    #[inline]
    pub fn append_val2<T: AsRef<Value>>(
        &mut self,
        v: T,
    ) -> bool {
        unsafe {
            match append_val(ALLOCATOR, self.0, v.as_ref()) {
                (vv, Some(lp)) => {
                    self.0 = lp;
                    true
                }
                (vv, None) => false
            }
        }
    }

    #[inline]
    pub fn append_val<'b>(
        &mut self,
        v: &'b Value,
    ) -> (&'b Value, bool) {
        unsafe {
            match append_val(ALLOCATOR, self.0, v) {
                (vv, Some(lp)) => {
                    self.0 = lp;
                    (vv, true)
                }
                (vv, None) => (vv, false)
            }
        }
    }

    #[inline]
    pub fn append_signed_int<T>(
        &mut self,
        v: T,
    ) -> bool
        where
            T: Int {
        unsafe {
            match append_signed_int(ALLOCATOR, self.0, v) {
                Some(lp) => {
                    self.0 = lp;
                    true
                }
                None => false
            }
        }
    }

    #[inline]
    pub fn append_string<T>(
        &mut self,
        v: T,
    ) -> bool
        where
            T: Str {
        unsafe {
            match append_string(ALLOCATOR, self.0, v) {
                Some(lp) => {
                    self.0 = lp;
                    true
                }
                None => false
            }
        }
    }

    #[inline]
    pub fn delete(
        &mut self,
        p: element,
    ) -> Option<element> {
        unsafe {
            match delete(ALLOCATOR, self.0, p) {
                Some((lp, ele)) => {
                    self.0 = lp;
                    Some(ele)
                }
                None => None
            }
        }
    }

    #[inline]
    pub fn get(&self, ele: element) -> Option<Value> {
        if ele.is_null() {
            None
        } else {
            Some(get(ele))
        }
    }

    #[inline(always)]
    pub fn get_i8(&self, ele: element) -> i8 {
        get_i8(ele)
    }

    #[inline(always)]
    pub fn get_u8(&self, ele: element) -> u8 {
        get_u8(ele)
    }

    #[inline(always)]
    pub fn get_i16(&self, ele: element) -> i16 {
        get_i16(ele)
    }

    #[inline(always)]
    pub fn get_u16(&self, ele: element) -> u16 {
        get_u16(ele)
    }

    #[inline(always)]
    pub fn get_i32(&self, ele: element) -> i32 {
        get_i32(ele)
    }

    #[inline(always)]
    pub fn get_u32(&self, ele: element) -> u32 {
        get_u32(ele)
    }

    #[inline(always)]
    pub fn get_i64(&self, ele: element) -> i64 {
        get_i64(ele)
    }

    #[inline(always)]
    pub fn get_u64(&self, ele: element) -> u64 {
        get_u64(ele)
    }

    #[inline(always)]
    pub fn get_i128(&self, ele: element) -> i128 {
        get_i128(ele)
    }

    #[inline(always)]
    pub fn get_u128(&self, ele: element) -> u128 {
        get_u128(ele)
    }

    #[inline(always)]
    pub fn get_f32(&self, ele: element) -> f32 {
        get_f32(ele)
    }

    #[inline(always)]
    pub fn get_f64(&self, ele: element) -> f64 {
        get_f64(ele)
    }

    #[inline(always)]
    pub fn get_isize(&self, ele: element) -> isize {
        get_isize(ele)
    }

    #[inline(always)]
    pub fn get_usize(&self, ele: element) -> usize {
        get_usize(ele)
    }

    #[inline(always)]
    pub fn get_int(&self, ele: element) -> i64 {
        get_int(ele)
    }

    #[inline(always)]
    pub fn get_signed_int(&self, ele: element) -> i64 {
        get_signed_int(ele)
    }

    #[inline(always)]
    pub fn get_str<'a>(&self, ele: element) -> &'a str {
        get_str(ele)
    }

    #[inline(always)]
    pub fn get_bytes<'a>(&self, ele: element) -> &'a [u8] {
        get_bytes(ele)
    }

    #[inline(always)]
    pub fn seek(&self, index: isize) -> Option<element> {
        seek(self.0, index)
    }

    #[inline(always)]
    pub fn first(&self) -> Option<element> {
        first(self.0)
    }

    #[inline(always)]
    pub fn last(&self) -> Option<element> {
        last(self.0)
    }

    #[inline(always)]
    pub fn next(&self, p: element) -> Option<element> {
        next(self.0, p)
    }

    #[inline(always)]
    pub fn prev(&self, p: element) -> Option<element> {
        prev(self.0, p)
    }

    #[inline(always)]
    pub fn iter<F>(&self, f: F) where F: Fn(element, Value) -> bool {
        iter(self.0, f)
    }

    #[inline(always)]
    pub fn iter_index<F>(&self, f: F) where F: Fn(usize, element, Value) -> bool {
        iter_index(self.0, f)
    }

    #[inline(always)]
    pub fn iter_rev<F>(&self, f: F) where F: Fn(element, Value) -> bool {
        iter_rev(self.0, f)
    }
}

impl Drop for Listpack {
    fn drop(&mut self) {
        println!("dropped Listpack");
//        if !self.0.is_null() {
        unsafe { ALLOCATOR.dealloc(self.0); }
//        }
    }
}

const INTBUF_SIZE: usize = 21;

pub const HDR_SIZE: isize = 6;
pub const HDR_NUMELE_UNKNOWN: u16 = u16::max_value();
pub const MAX_BACKLEN_SIZE: usize = 5;
pub const MAX_ENTRY_BACKLEN: u64 = 34359738367;
pub const EOF: u8 = 0xFF;


pub const HDR_USIZE: usize = 6;
const MAX_INT_ENCODING_LEN: usize = 9;
const HDR_NUMELE_UNKNOWN_ISIZE: isize = u16::max_value() as isize;

const ENCODING_7BIT_UINT: u8 = 0;
const ENCODING_7BIT_UINT_MASK: u8 = 0x80;

const ENCODING_6BIT_STR: u8 = 0x80;
const ENCODING_6BIT_STR_MASK: u8 = 0xC0;

const ENCODING_13BIT_INT: u8 = 0xC0;
const ENCODING_13BIT_INT_MASK: u8 = 0xE0;

const ENCODING_12BIT_STR: u8 = 0xE0;
const ENCODING_12BIT_STR_MASK: u8 = 0xF0;

const ENCODING_16BIT_INT_MASK: u8 = 0xFF;
const ENCODING_16BIT_INT: u8 = 0xF1;

const ENCODING_24BIT_INT: u8 = 0xF2;
const ENCODING_24BIT_INT_MASK: u8 = 0xFF;

const ENCODING_32BIT_INT: u8 = 0xF3;
const ENCODING_32BIT_INT_MASK: u8 = 0xFF;

const ENCODING_64BIT_INT: u8 = 0xF4;
const ENCODING_64BIT_INT_MASK: u8 = 0xFF;

const ENCODING_32BIT_STR: u8 = 0xF0;
const ENCODING_32BIT_STR_MASK: u8 = 0xFF;


#[inline(always)]
pub fn is_7bit_uint(b: u8) -> bool {
    b & ENCODING_7BIT_UINT_MASK == ENCODING_7BIT_UINT
}

#[inline(always)]
pub fn is_6bit_str(b: u8) -> bool {
    b & ENCODING_6BIT_STR_MASK == ENCODING_6BIT_STR
}

#[inline(always)]
pub fn is_13bit_int(b: u8) -> bool {
    b & ENCODING_13BIT_INT_MASK == ENCODING_13BIT_INT
}

#[inline(always)]
pub fn is_12bit_str(b: u8) -> bool {
    b & ENCODING_12BIT_STR_MASK == ENCODING_12BIT_STR
}

#[inline(always)]
pub fn is_16bit_int(b: u8) -> bool {
    b & ENCODING_16BIT_INT_MASK == ENCODING_16BIT_INT
}

#[inline(always)]
pub fn is_24bit_int(b: u8) -> bool {
    b & ENCODING_24BIT_INT_MASK == ENCODING_24BIT_INT
}

#[inline(always)]
pub fn is_32bit_int(b: u8) -> bool {
    b & ENCODING_32BIT_INT_MASK == ENCODING_32BIT_INT
}

#[inline(always)]
pub fn is_32bit_str(b: u8) -> bool {
    b & ENCODING_32BIT_STR_MASK == ENCODING_32BIT_STR
}

#[inline(always)]
pub fn is_64bit_int(b: u8) -> bool {
    b & ENCODING_64BIT_INT_MASK == ENCODING_64BIT_INT
}


#[inline(always)]
pub fn str_len_6bit(p: *mut u8) -> u32 {
    unsafe { ((*p) & 0x3Fu8) as u32 }
}

#[inline(always)]
pub fn str_len_12bit(p: *mut u8) -> u32 {
    unsafe {
        u32::from_le(
            ((*p) & 0xFu8) as u32 | (*p.offset(1)) as u32
        )
    }
}

#[inline(always)]
pub fn str_len_32bit(p: *mut u8) -> u32 {
    unsafe {
        u32::from_le(((*p.offset(1)) as u32) << 0 |
            ((*p.offset(2)) as u32) << 8 |
            ((*p.offset(3)) as u32) << 16 |
            ((*p.offset(4)) as u32) << 24)
    }
}

#[inline(always)]
pub fn get_total_bytes(p: *mut u8) -> u32 {
    unsafe {
        u32::from_le(((*p.offset(0) as u32) << 0) |
            (*p.offset(1) as u32) << 8 |
            (*p.offset(2) as u32) << 16 |
            (*p.offset(3) as u32) << 24)
    }
}

#[inline(always)]
pub fn set_total_bytes(p: *mut u8, v: u32) {
    unsafe {
        *p = (v & 0xff) as u8;
        *p.offset(1) = (v >> 8 & 0xff) as u8;
        *p.offset(2) = (v >> 16 & 0xff) as u8;
        *p.offset(3) = (v >> 24 & 0xff) as u8;
    }
}

#[inline(always)]
pub fn get_num_elements(p: *mut u8) -> u16 {
    unsafe {
        u16::from_le(((*p.offset(4) as u16) << 0) |
            (*p.offset(5) as u16) << 8)
    }
}

#[inline(always)]
pub fn set_num_elements(p: *mut u8, v: u16) {
    unsafe {
        *p.offset(4) = (v & 0xff) as u8;
        *p.offset(5) = (v >> 8 & 0xff) as u8;
    }
}

impl Into<Value> for super::sds::Sds {
    #[inline]
    fn into(self) -> Value {
        sds_to_value(self)
    }
}

#[inline]
pub fn sds_to_value(mut s: super::sds::Sds) -> Value {
    unsafe {
        let mut value: i64 = 0;
        let size = super::sds::get_len(s);
        if super::sds::string2ll(s, size, (&mut value) as *mut i64) == 0 {
            Value::String(s as *const u8, size as u32)
        } else {
            Value::Int(value)
        }
    }
}

#[inline]
pub fn parse_raw(mut p: *const u8, size: usize) -> Value {
    unsafe {
        let mut value: i64 = 0;
        if super::sds::string2ll(p as *mut i8, size, (&mut value) as *mut i64) == 0 {
            Value::String(p as *const u8, size as u32)
        } else {
            Value::Int(value)
        }
    }
}

//#[inline(always)]
//pub fn set_total_bytes

pub fn encode_int(mut v: i64, intenc: &mut [u8; MAX_INT_ENCODING_LEN]) -> u64 {
    v = v.to_le();

    if v >= 0 && v <= 127 {
        // Single byte 0-127 integer
        intenc[0] = v as u8;
        1
    } else if v >= -4096 && v <= 4095 {
        // 13 bit integer
        if v < 0 {
            v = (1i64 << 13) + v;
        }

        intenc[0] = (v >> 8) as u8 | ENCODING_13BIT_INT;
        intenc[1] = (v & 0xff) as u8;

        2
    } else if v >= -32768 && v <= 32767 {
        // 16 bit integer
        if v < 0 {
            v = (1i64 << 16) + v;
        }

        intenc[0] = ENCODING_16BIT_INT;
        intenc[1] = (v & 0xff) as u8;
        intenc[2] = (v >> 8) as u8;
        3
    } else if v >= -8388608 && v <= 8388607 {
        // 24 bit integer
        if v < 0 {
            v = (1i64 << 24) + v;
        }

        intenc[0] = ENCODING_24BIT_INT;
        intenc[1] = (v & 0xff) as u8;
        intenc[2] = ((v >> 8) & 0xff) as u8;
        intenc[3] = (v >> 16) as u8;
        4
    } else if v >= -2147483648 && v <= 2147483647 {
        // 32 bit integer
        if v < 0 {
            v = (1i64 << 32) + v;
        }

        intenc[0] = ENCODING_32BIT_INT;
        intenc[1] = (v & 0xff) as u8;
        intenc[2] = ((v >> 8) & 0xff) as u8;
        intenc[3] = ((v >> 16) & 0xff) as u8;
        intenc[4] = (v >> 24) as u8;
        5
    } else {
        // 64 bit integer
        let uv = v as u64;
        intenc[0] = ENCODING_64BIT_INT;
        intenc[1] = (uv & 0xff) as u8;
        intenc[2] = ((uv >> 8) & 0xff) as u8;
        intenc[3] = ((uv >> 16) & 0xff) as u8;
        intenc[4] = ((uv >> 24) & 0xff) as u8;
        intenc[5] = ((uv >> 32) & 0xff) as u8;
        intenc[6] = ((uv >> 40) & 0xff) as u8;
        intenc[7] = ((uv >> 48) & 0xff) as u8;
        intenc[8] = (uv >> 56) as u8;
        9
    }
}

#[inline(always)]
pub fn string_front_len(size: u32) -> u64 {
    if size < 64 {
        1
    } else if size < 4096 {
        2
    } else {
        5
    }
}

fn encode_string(buf: *mut u8, ele: *const u8, size: u32) -> u32 {
    unsafe {
        if size < 64 {
            *buf.offset(0) = size as u8 | ENCODING_6BIT_STR;
            ptr::copy_nonoverlapping(
                ele as *const u8,
                buf.offset(1),
                size as usize,
            );
            1 + size
        } else if size < 4096 {
            *buf.offset(0) = (size >> 8) as u8 | ENCODING_12BIT_STR;
            *buf.offset(1) = (size & 0xff) as u8;
            ptr::copy_nonoverlapping(
                ele as *const u8,
                buf.offset(2),
                size as usize,
            );
            2 + size
        } else {
            *buf.offset(0) = ENCODING_32BIT_STR;
            *buf.offset(1) = (size & 0xff) as u8;
            *buf.offset(2) = ((size >> 8) & 0xff) as u8;
            *buf.offset(3) = ((size >> 16) & 0xff) as u8;
            *buf.offset(4) = ((size >> 24) & 0xff) as u8;
            ptr::copy_nonoverlapping(
                ele as *const u8,
                buf.offset(5),
                size as usize,
            );
            5 + size
        }
    }
}

/// Store a reverse-encoded variable length field, representing the length
/// of the previous element of size 'l', in the target buffer 'buf'.
/// The function returns the number of bytes used to encode it, from
/// 1 to 5. If 'buf' is NULL the function just returns the number of bytes
/// needed in order to encode the backlen.
pub fn encode_backlen(buf: &mut [u8; MAX_BACKLEN_SIZE], l: u64) -> usize {
    if l <= 127 {
        buf[0] = l as u8;
        return 1;
    } else if l < 16383 {
        buf[0] = (l >> 7) as u8;
        buf[1] = (l & 127) as u8 | 128u8;
        return 2;
    } else if l < 2097151 {
        buf[0] = (l >> 14) as u8;
        buf[1] = ((l >> 7) & 127) as u8 | 128u8;
        buf[2] = (l & 127) as u8 | 128u8;
        return 3;
    } else if l < 268435455 {
        buf[0] = (l >> 21) as u8;
        buf[1] = ((l >> 14) & 127) as u8 | 128u8;
        buf[2] = ((l >> 7) & 127) as u8 | 128u8;
        buf[3] = (l & 127) as u8 | 128u8;
        return 4;
    } else {
        buf[0] = (l >> 28) as u8;
        buf[1] = ((l >> 21) & 127) as u8 | 128u8;
        buf[2] = ((l >> 14) & 127) as u8 | 128u8;
        buf[3] = ((l >> 7) & 127) as u8 | 128u8;
        buf[4] = (l & 127) as u8 | 128u8;
        return 5;
    }
}

pub fn backlen_size(l: u64) -> u32 {
    if l <= 127 {
        return 1;
    } else if l < 16383 {
        return 2;
    } else if l < 2097151 {
        return 3;
    } else if l < 268435455 {
        return 4;
    } else {
        return 5;
    }
}


pub fn decode_backlen(mut buf: *mut u8) -> u64 {
    unsafe {
        let mut val = 0u64;
        let mut shift = 0u64;
        loop {
            val |= ((*buf.offset(0)) as u64 & 127u64) << shift;
            if (*buf.offset(0)) as u64 & 128u64 == 0 {
                break;
            }
            shift += 7;
            buf = buf.offset(-1);
            if shift > 28 {
                return u64::max_value();
            }
        }
        val
    }
}

#[inline(always)]
pub fn is_valid_element(lp: listpack, ele: element, len: usize) -> bool {
    if lp.is_null() || ele.is_null() {
        return false;
    }

    let lp_uintptr = lp as usize;
    let ele_uintptr = ele as usize;

    ele_uintptr >= lp_uintptr && ele_uintptr < (lp_uintptr + len)
}

pub fn get(p: element) -> Value {
    unsafe {
        let mut val: i64;
        let mut uval: u64;
        let negstart: u64;
        let negmax: u64;

        if is_7bit_uint(*p) {
            negstart = u64::max_value();
            negmax = 0;
            uval = (*p & 0x7f) as u64;
        } else if is_6bit_str(*p) {
            return Value::String(
                p.offset(1),
                str_len_6bit(p),
            );
        } else if is_13bit_int(*p) {
            uval = ((*p as u64 & 0x1f) << 8) | (*p.offset(1) as u64);
            negstart = 1u64 << 12;
            negmax = 8191;
        } else if is_16bit_int(*p) {
            uval = (*p.offset(1) as u64) |
                (*p.offset(2) as u64) << 8;
            negstart = 1u64 << 15;
            negmax = u16::max_value() as u64;
        } else if is_24bit_int(*p) {
            uval = (*p.offset(1) as u64) |
                (*p.offset(2) as u64) << 8 |
                (*p.offset(3) as u64) << 16;
            negstart = 1u64 << 23;
            negmax = (u32::max_value() >> 8) as u64;
        } else if is_32bit_int(*p) {
            uval = (*p.offset(1) as u64) |
                (*p.offset(2) as u64) << 8 |
                (*p.offset(3) as u64) << 16 |
                (*p.offset(4) as u64) << 24;
            negstart = 1u64 << 31;
            negmax = u32::max_value() as u64;
        } else if is_64bit_int(*p) {
            uval = (*p.offset(1) as u64) |
                (*p.offset(2) as u64) << 8 |
                (*p.offset(3) as u64) << 16 |
                (*p.offset(4) as u64) << 24 |
                (*p.offset(5) as u64) << 32 |
                (*p.offset(6) as u64) << 40 |
                (*p.offset(7) as u64) << 48 |
                (*p.offset(8) as u64) << 56;
            negstart = 1u64 << 63;
            negmax = u64::max_value();
        } else if is_12bit_str(*p) {
            return Value::String(
                p.offset(2),
                str_len_12bit(p),
            );
        } else if is_32bit_str(*p) {
            return Value::String(
                p.offset(5),
                str_len_32bit(p),
            );
        } else {
            uval = 12345678900000000_u64 + *p as u64;
            negstart = u64::max_value();
            negmax = 0;
        }

        // We reach this code path only for integer encodings.
        // Convert the unsigned value to the signed one using two's complement
        // rule.
        if uval >= negstart {
            // This three steps conversion should avoid undefined behaviors
            // in the unsigned -> signed conversion.
            uval = negmax - uval;
            val = uval as i64;
            val = -val - 1;
        } else {
            val = uval as i64;
        }

        Value::Int(val)
    }
}

#[inline(always)]
pub fn get_i8(ele: element) -> i8 {
    i8::from(get(ele))
}

#[inline(always)]
pub fn get_u8(ele: element) -> u8 {
    u8::from(get(ele))
}

#[inline(always)]
pub fn get_i16(ele: element) -> i16 {
    i16::from(get(ele))
}

#[inline(always)]
pub fn get_u16(ele: element) -> u16 {
    u16::from(get(ele))
}

#[inline(always)]
pub fn get_i32(ele: element) -> i32 {
    i32::from(get(ele))
}

#[inline(always)]
pub fn get_u32(ele: element) -> u32 {
    u32::from(get(ele))
}

#[inline(always)]
pub fn get_i64(ele: element) -> i64 {
    i64::from(get(ele))
}

#[inline(always)]
pub fn get_u64(ele: element) -> u64 {
    u64::from(get(ele))
}

#[inline(always)]
pub fn get_i128(ele: element) -> i128 {
    i128::from(get(ele))
}

#[inline(always)]
pub fn get_u128(ele: element) -> u128 {
    u128::from(get(ele))
}

#[inline(always)]
pub fn get_f32(ele: element) -> f32 {
    f32::from(get(ele))
}

#[inline(always)]
pub fn get_f64(ele: element) -> f64 {
    f64::from(get(ele))
}

#[inline(always)]
pub fn get_isize(ele: element) -> isize {
    isize::from(get(ele))
}

#[inline(always)]
pub fn get_usize(ele: element) -> usize {
    usize::from(get(ele))
}

#[inline(always)]
pub fn get_int(ele: element) -> i64 {
    i64::from(get(ele))
}

#[inline(always)]
pub fn get_signed_int(ele: element) -> i64 {
    zigzag(i64::from(get(ele)))
}

#[inline(always)]
pub fn get_str<'a>(ele: element) -> &'a str {
    get(ele).into()
}

#[inline(always)]
pub fn get_bytes<'a>(ele: element) -> &'a [u8] {
    get(ele).into()
}

/// Return the encoded length of the listpack element pointed by 'p'. If the
/// element encoding is wrong then 0 is returned.
pub fn get_encoded_size(p: element) -> u32 {
    unsafe {
        if is_7bit_uint(*p) {
            return 1;
        }
        if is_6bit_str(*p) {
            return 1 + str_len_6bit(p);
        }
        if is_13bit_int(*p) {
            return 2;
        }
        if is_16bit_int(*p) {
            return 3;
        }
        if is_24bit_int(*p) {
            return 4;
        }
        if is_32bit_int(*p) {
            return 5;
        }
        if is_64bit_int(*p) {
            return 9;
        }
        if is_12bit_str(*p) {
            return 2 + str_len_12bit(p);
        }
        if is_32bit_str(*p) {
            return 5 + str_len_32bit(p);
        }
        if *p == EOF {
            return 1;
        }
        0
    }
}

/// Return the number of elements inside the listpack. This function attempts
/// to use the cached value when within range, otherwise a full scan is
/// needed. As a side effect of calling this function, the listpack header
/// could be modified, because if the count is found to be already within
/// the 'numele' header field range, the new value is set.
#[inline(always)]
pub fn length(lp: listpack) -> u32 {
    let numele = get_num_elements(lp);
    if numele != HDR_NUMELE_UNKNOWN {
        numele as u32
    } else {
        // Too many elements inside the listpack. We need to scan in order
        // to get the total number.
        let mut count: u32 = 0;
        match first(lp) {
            Some(mut p) => {
                count = count + 1;
                while let Some(ele) = next(lp, p) {
                    p = ele;
                    count = count + 1;
                }
            }
            None => {
                set_num_elements(lp, 0)
            }
        }

        // If the count is again within range of the header numele field, set it.
        if count < HDR_NUMELE_UNKNOWN as u32 {
            set_num_elements(lp, count as u16);
        }

        count
    }
}

impl<'a> Into<&'a str> for Value {
    #[inline]
    fn into(self) -> &'a str {
        self.as_str()
    }
}

impl<'a> Into<&'a [u8]> for Value {
    #[inline]
    fn into(self) -> &'a [u8] {
        self.as_bytes()
    }
}


impl Value {
    #[inline]
    pub fn as_bytes<'a>(&self) -> &'a [u8] {
        match *self {
            Value::Int(v) => {
                unsafe {
                    std::slice::from_raw_parts(
                        &v as *const _ as *const u8,
                        std::mem::size_of::<i64>(),
                    )
                }
            }
            Value::String(ptr, len) => {
                if ptr.is_null() || len == 0 {
                    EMPTY
                } else {
                    unsafe {
                        std::slice::from_raw_parts(ptr, len as usize)
                    }
                }
            }
        }
    }

    #[inline]
    pub fn as_str<'a>(&self) -> &'a str {
        match *self {
            Value::Int(v) => {
                unsafe {
                    std::str::from_utf8_unchecked(
                        std::slice::from_raw_parts(
                            &v as *const _ as *const u8,
                            std::mem::size_of::<i64>(),
                        )
                    )
                }
            }
            Value::String(ptr, len) => {
                unsafe {
                    std::str::from_utf8_unchecked(
                        std::slice::from_raw_parts(ptr, len as usize)
                    )
                }
            }
        }
    }

    #[inline(always)]
    pub fn encoded_size(&self) -> u32 {
        match *self {
            Value::Int(v) => {
                if v >= 0 && v <= 127 {
                    1
                } else if v >= -4096 && v <= 4095 {
                    2
                } else if v >= -32768 && v <= 32767 {
                    3
                } else if v >= -8388608 && v <= 8388607 {
                    // 24 bit integer
                    4
                } else if v >= -2147483648 && v <= 2147483647 {
                    // 32 bit integer
                    5
                } else {
                    // 64 bit integer
                    9
                }
            }
            Value::String(_, size) => {
                if size < 64 {
                    1 + size
                } else if size < 4096 {
                    2 + size
                } else {
                    5 + size
                }
            }
        }
    }

    #[inline(always)]
    pub fn encoded_sizes(&self) -> (u32, u32) {
        let size = self.encoded_size();
        (size, backlen_size(size as u64))
    }

    #[inline(always)]
    pub fn size_for_write(&self) -> u32 {
        let size = self.encoded_size();
        size + backlen_size(size as u64)
    }

    #[inline(always)]
    unsafe fn encode_backlen(buf: *mut u8, l: u32) {
        if l <= 127 {
            *buf = l as u8;
        } else if l < 16383 {
            *buf = (l >> 7) as u8;
            *buf.offset(1) = (l & 127) as u8 | 128u8;
        } else if l < 2097151 {
            *buf = (l >> 14) as u8;
            *buf.offset(1) = ((l >> 7) & 127) as u8 | 128u8;
            *buf.offset(2) = (l & 127) as u8 | 128u8;
        } else if l < 268435455 {
            *buf = (l >> 21) as u8;
            *buf.offset(1) = ((l >> 14) & 127) as u8 | 128u8;
            *buf.offset(2) = ((l >> 7) & 127) as u8 | 128u8;
            *buf.offset(3) = (l & 127) as u8 | 128u8;
        } else {
            *buf = (l >> 28) as u8;
            *buf.offset(1) = ((l >> 21) & 127) as u8 | 128u8;
            *buf.offset(2) = ((l >> 14) & 127) as u8 | 128u8;
            *buf.offset(3) = ((l >> 7) & 127) as u8 | 128u8;
            *buf.offset(4) = (l & 127) as u8 | 128u8;
        }
    }

    #[inline(always)]
    unsafe fn encode(&self, dst: *mut u8, encoded_size: u32) {
        match *self {
            Value::Int(mut v) => {
                // Little Endian
                v = v.to_le();

                // Match on the encoded size which is the tag, value and backlen.
                match encoded_size {
                    2 => {
                        // Single byte 0-127 integer
                        *dst = v as u8;
                        // Encode backlen
                        *dst.offset(1) = 1u8;
                    }
                    3 => {
                        // 13 bit integer
                        if v < 0 {
                            v = (1i64 << 13) + v;
                        }

                        *dst = (v >> 8) as u8 | ENCODING_13BIT_INT;
                        *dst.offset(1) = (v & 0xff) as u8;
                        // Encode backlen
                        *dst.offset(2) = 2u8;
                    }
                    4 => {
                        // 16 bit integer
                        if v < 0 {
                            v = (1i64 << 16) + v;
                        }

                        *dst = ENCODING_16BIT_INT;
                        *dst.offset(1) = (v & 0xff) as u8;
                        *dst.offset(2) = (v >> 8) as u8;
                        // Encode backlen
                        *dst.offset(3) = 3u8;
                    }
                    5 => {
                        // 24 bit integer
                        if v < 0 {
                            v = (1i64 << 24) + v;
                        }

                        *dst = ENCODING_24BIT_INT;
                        *dst.offset(1) = (v & 0xff) as u8;
                        *dst.offset(2) = ((v >> 8) & 0xff) as u8;
                        *dst.offset(3) = (v >> 16) as u8;
                        // Encode backlen
                        *dst.offset(4) = 4u8;
                    }
                    6 => {
                        // 32 bit integer
                        if v < 0 {
                            v = (1i64 << 32) + v;
                        }

                        *dst = ENCODING_32BIT_INT;
                        *dst.offset(1) = (v & 0xff) as u8;
                        *dst.offset(2) = ((v >> 8) & 0xff) as u8;
                        *dst.offset(3) = ((v >> 16) & 0xff) as u8;
                        *dst.offset(4) = (v >> 24) as u8;
                        // Encode backlen
                        *dst.offset(5) = 5u8;
                    }
                    10 => {
                        // 64 bit integer
                        let uv = v as u64;
                        *dst = ENCODING_64BIT_INT;
                        *dst.offset(1) = (uv & 0xff) as u8;
                        *dst.offset(2) = ((uv >> 8) & 0xff) as u8;
                        *dst.offset(3) = ((uv >> 16) & 0xff) as u8;
                        *dst.offset(4) = ((uv >> 24) & 0xff) as u8;
                        *dst.offset(5) = ((uv >> 32) & 0xff) as u8;
                        *dst.offset(6) = ((uv >> 40) & 0xff) as u8;
                        *dst.offset(7) = ((uv >> 48) & 0xff) as u8;
                        *dst.offset(8) = (uv >> 56) as u8;
                        // Encode backlen
                        *dst.offset(9) = 9u8;
                    }
                    _ => {
                        // NOOP
                    }
                }
            }
            Value::String(ele, size) => {
                if size < 64 {
                    *dst.offset(0) = size as u8 | ENCODING_6BIT_STR;
                    ptr::copy_nonoverlapping(
                        ele as *const u8,
                        dst.offset(1),
                        size as usize,
                    );
                    // Encode backlen
                    Value::encode_backlen(dst.offset(encoded_size as isize), size);
                } else if size < 4096 {
                    *dst.offset(0) = (size >> 8) as u8 | ENCODING_12BIT_STR;
                    *dst.offset(1) = (size & 0xff) as u8;
                    ptr::copy_nonoverlapping(
                        ele as *const u8,
                        dst.offset(2),
                        size as usize,
                    );
                    // Encode backlen
                    Value::encode_backlen(dst.offset(encoded_size as isize), size);
                } else {
                    *dst.offset(0) = ENCODING_32BIT_STR;
                    *dst.offset(1) = (size & 0xff) as u8;
                    *dst.offset(2) = ((size >> 8) & 0xff) as u8;
                    *dst.offset(3) = ((size >> 16) & 0xff) as u8;
                    *dst.offset(4) = ((size >> 24) & 0xff) as u8;
                    ptr::copy_nonoverlapping(
                        ele as *const u8,
                        dst.offset(5),
                        size as usize,
                    );
                    // Encode backlen
                    Value::encode_backlen(dst.offset(encoded_size as isize), size);
                }
            }
        }
    }
}

#[inline(always)]
pub fn first(lp: listpack) -> Option<element> {
    unsafe {
        let l = lp.offset(HDR_SIZE);
        if *l == EOF {
            None
        } else {
            Some(l)
        }
    }
}

#[inline(always)]
pub fn last(lp: listpack) -> Option<element> {
    unsafe {
        let p = lp.offset(get_total_bytes(lp) as isize - 1);
        prev(lp, p)
    }
}

/// Skip the current entry returning the next. It is invalid to call this
/// function if the current element is the EOF element at the end of the
/// listpack, however, while this function is used to implement lpNext(),
/// it does not return NULL when the EOF element is encountered.
#[inline(always)]
pub fn skip(p: element) -> element {
    unsafe {
        let mut entrylen = get_encoded_size(p);
        entrylen = entrylen + backlen_size(entrylen as u64);
        p.offset(entrylen as isize)
    }
}

/// If 'p' points to an element of the listpack, calling lpNext() will return
/// the pointer to the next element (the one on the right), or NULL if 'p'
/// already pointed to the last element of the listpack.
#[inline(always)]
pub fn next(_lp: listpack, mut p: *mut u8) -> Option<element> {
    unsafe {
        p = skip(p);
        if *p == EOF {
            None
        } else {
            Some(p)
        }
    }
}

/// If 'p' points to an element of the listpack, calling lpPrev() will return
/// the pointer to the previous element (the one on the left), or NULL if 'p'
/// already pointed to the first element of the listpack.
#[inline(always)]
pub fn prev(lp: listpack, mut p: element) -> Option<element> {
    unsafe {
        if ((p as usize) - (lp as usize)) <= HDR_USIZE {
            None
        } else {
            p = p.offset(-1);

            let mut prevlen = decode_backlen(p);
            prevlen += backlen_size(prevlen) as u64;

            Some(p.offset((-(prevlen as isize)) + 1))
        }
    }
}

/// If 'p' points to an element of the listpack, calling prev_no_hdr() will return
/// the pointer to the previous element (the one on the left), or None if 'p'
/// already pointed to the first element of the listpack.
#[inline(always)]
pub fn prev_no_hdr(lp: listpack, mut p: element) -> Option<element> {
    unsafe {
        let p_uintptr = p as usize;
        let lp_uintptr = lp as usize;
        if p_uintptr < lp_uintptr {
            None
        } else {
            p = p.offset(-1);

            let mut prevlen = decode_backlen(p);
            if prevlen >= u32::max_value() as u64 {
                return None;
            }
            prevlen += backlen_size(prevlen) as u64;

            Some(p.offset((-(prevlen as isize)) + 1))
        }
    }
}


/// Seek the specified element and returns the pointer to the seeked element.
/// Positive indexes specify the zero-based element to seek from the head to
/// the tail, negative indexes specify elements starting from the tail, where
/// -1 means the last element, -2 the penultimate and so forth. If the index
/// is out of range, NULL is returned.
pub fn seek(lp: listpack, mut index: isize) -> Option<element> {
    let mut forward = true;

    // We want to seek from left to right or the other way around
    // depending on the listpack length and the element position.
    // However if the listpack length cannot be obtained in constant time,
    // we always seek from left to right.
    let numele = get_num_elements(lp) as isize;
    if numele != HDR_NUMELE_UNKNOWN_ISIZE {
        if index < 0 {
            index = numele + index;
        }
        // Index still < 0 means out of range.
        if index < 0 {
            return None;
        }
        // Out of range the other side.
        if index >= numele {
            return None;
        }
        // We want to scan right-to-left if the element we are looking for
        // is past the half of the listpack.
        if index > numele / 2 {
            forward = false;
            // Left to right scanning always expects a negative index. Convert
            // our index to negative form.
            index = index - numele;
        }
    } else {
        // If the listpack length is unspecified, for negative indexes we
        // want to always scan left-to-right.
        if index < 0 {
            forward = false;
        }
    }

    // Forward and backward scanning is trivially based on next()/prev()
    if forward {
        match first(lp) {
            Some(mut ele) => {
                while index > 0 {
                    if let Some(e) = next(lp, ele) {
                        ele = e;
                        index = index - 1;
                    } else {
                        return None;
                    }
                }
                Some(ele)
            }
            None => None
        }
    } else {
        match last(lp) {
            Some(mut ele) => {
                while index < -1 {
                    if let Some(e) = prev(lp, ele) {
                        ele = e;
                        index = index + 1;
                    } else {
                        return None;
                    }
                }
                Some(ele)
            }
            None => None
        }
    }
}

///
#[inline]
pub fn new<'a, A>(allocator: &'a A) -> listpack where A: Allocator {
    let lp = allocator.alloc(HDR_USIZE + 1);
    set_total_bytes(lp, HDR_USIZE as u32 + 1);
    lp
}

#[inline]
pub fn zigzag(n: i64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}

//===----------------------------------------------------------------------===//
// Insert
//===----------------------------------------------------------------------===//

/// Insert a new element into the listpack.
#[inline]
pub fn insert<'a, A>(
    allocator: &'a A,
    mut lp: listpack,
    v: Value,
    place: Placement,
    target: element,
) -> Option<(listpack, element)>
    where A: Allocator {
    unsafe {
        let encoded_size = v.size_for_write();

        // Calculate the old and new sizes.
        let old_listpack_bytes = get_total_bytes(lp);
        let new_listpack_bytes = old_listpack_bytes + encoded_size;

        // Is it over the max size?
        if new_listpack_bytes > u32::max_value() {
            return None;
        }

        // Find target
        let mut p =
            match place {
                Placement::Before => {
                    // Gracefully handle null pointer and EOF as an append.
                    if target.is_null() || *target == EOF {
                        match append(allocator, lp, v) {
                            Some(new_lp) => return Some((
                                new_lp,
                                last(new_lp).unwrap_or(std::ptr::null_mut())
                            )),
                            None => return None
                        }
                    } else {
                        // Target is already what we want.
                        target
                    }
                }
                Placement::After => {
                    // Gracefully handle null pointer and EOF as an append.
                    if target.is_null() || *target == EOF {
                        match append(allocator, lp, v) {
                            Some(new_lp) => return Some((
                                new_lp,
                                last(new_lp).unwrap_or(std::ptr::null_mut())
                            )),
                            None => return None
                        }
                    } else {
                        // Find next element so we can place it before.
                        match next(lp, target) {
                            Some(ele) => ele,
                            None => match append(allocator, lp, v) {
                                Some(new_lp) => return Some((
                                    new_lp,
                                    last(new_lp).unwrap_or(std::ptr::null_mut())
                                )),
                                None => return None
                            }
                        }
                    }
                }
            };

        if p.is_null() {
            return None;
        }

        // Store the offset of the element 'p', so that we can obtain it's
        // address again after a reallocation.
        let poff = (p as usize) - (lp as usize);

        // realloc to make room
        lp = allocator.realloc(lp, new_listpack_bytes as usize);
        if lp.is_null() {
            return None;
        }

        p = lp.offset(poff as isize);

        // Setup the listpack relocating the elements to make the exact room
        // we need to store the new one.
        std::ptr::copy(
            p,
            p.offset(encoded_size as isize),
            old_listpack_bytes as usize - poff,
        );

        // Write value.
        // This overwrites the EOF byte at the end which will get added
        // immediately after this new value.
        v.encode(p, encoded_size);

        // Write EOF
        *lp.offset(new_listpack_bytes as isize - 1) = EOF;

        // Update header
        let num_elements = get_num_elements(lp);
        if num_elements != HDR_NUMELE_UNKNOWN {
            set_num_elements(lp, num_elements + 1);
        }
        set_total_bytes(lp, new_listpack_bytes);

        Some((lp, p))
    }
}

///
#[inline]
pub fn insert_int<'a, A, I>(
    allocator: &'a A,
    mut lp: listpack,
    v: I,
    place: Placement,
    target: element,
) -> Option<(listpack, element)>
    where
        A: Allocator,
        I: Int {
    insert(allocator, lp, Value::Int(v.to_int64()), place, target)
}

///
#[inline]
pub fn insert_signed_int<'a, A, I>(
    allocator: &'a A,
    mut lp: listpack,
    v: I,
    place: Placement,
    target: element,
) -> Option<(listpack, element)>
    where
        A: Allocator,
        I: Int {
    insert(allocator, lp, Value::Int(zigzag(v.to_int64())), place, target)
}

///
#[inline]
pub fn insert_string<'a, A, S>(
    allocator: &'a A,
    mut lp: listpack,
    mut v: S,
    place: Placement,
    target: element,
) -> Option<(listpack, element)>
    where
        A: Allocator,
        S: Str {
    insert(allocator, lp, v.as_value(), place, target)
}


//===----------------------------------------------------------------------===//
// Replace
//===----------------------------------------------------------------------===//

///
#[inline]
pub fn replace<'a, A>(
    allocator: &'a A,
    mut lp: listpack,
    mut p: element,
    v: Value,
) -> Option<(listpack, element)>
    where A: Allocator {
    unsafe {
        // Let's try to be somewhat safe.
        if lp.is_null() || p.is_null() || *p == EOF {
            return None;
        }

        let encoded_size = v.size_for_write();
        let old_value = get(p);
        let old_size = old_value.size_for_write();

        if old_size == encoded_size {
            return Some((lp, p));
        }

        // Calculate size delta.
        let delta = (encoded_size as isize) - (old_size as isize);
        if delta == 0 {
            // Same size!
            // Encode in place.
            v.encode(p, encoded_size);
            return Some((lp, p));
        }

        // Store the offset of the element 'p', so that we can obtain it's
        // address again after a reallocation.
        let p_uintptr = p as usize;
        let lp_uintptr = lp as usize;

        // Bounds check
        if p_uintptr <= lp_uintptr {
            // Whoops!!! "p" is not within this listpack!
            return None;
        }
        // Calculate "p" offset.
        let poff = p_uintptr - lp_uintptr;

        // Calculate the old and new sizes.
        let old_listpack_bytes = get_total_bytes(lp);
        let new_listpack_bytes = (old_listpack_bytes as isize + delta) as usize;

        // Bounds check
        if poff > new_listpack_bytes {
            // Whoops!!! "p" is not within this listpack!
            return None;
        }

        // Is it over the max size?
        if new_listpack_bytes > u32::max_value() as usize {
            return None;
        }

        if delta > 0 {
            // Shift next element by delta bytes.
            match next(lp, p) {
                Some(ele) => {
                    // Calculate the next element offset.
                    let ele_uintptr = ele as usize;
                    if ele_uintptr <= p_uintptr {
                        return None;
                    }
                    let eleoff = ele_uintptr - lp_uintptr;

                    // Grow allocation. We must do this before the shift since
                    // it could potentially overflow the actual allocation.
                    lp = allocator.realloc(lp, new_listpack_bytes);
                    if lp.is_null() {
                        return None;
                    }

                    // Update "p" to it's pointer in the potentially new allocation.
                    p = lp.offset(poff as isize);

                    std::ptr::copy(
                        lp.offset(eleoff as isize),
                        lp.offset((eleoff as isize) + delta),
                        old_listpack_bytes as usize - eleoff,
                    );
                }
                None => {
                    // Grow allocation. We must do this before the shift since
                    // it could potentially overflow the actual allocation.
                    lp = allocator.realloc(lp, new_listpack_bytes);
                    if lp.is_null() {
                        return None;
                    }

                    // Update "p" to it's pointer in the potentially new allocation.
                    p = lp.offset(poff as isize);

                    // Treat it as EOF
                    v.encode(p, encoded_size);

                    // Write EOF
                    *lp.offset((new_listpack_bytes as isize) - 1) = EOF;
                }
            }
        } else if delta < 0 {
            // Shift next element by back byte "delta" bytes.
            // We must do this before resizing the allocation since it would
            // result in data loss.
            match next(lp, p) {
                Some(ele) => {
                    let ele_uintptr = ele as usize;
                    if ele_uintptr <= p_uintptr {
                        return None;
                    }

                    let eleoff = ele_uintptr - lp_uintptr;

                    std::ptr::copy(
                        lp.offset(eleoff as isize),
                        lp.offset((eleoff as isize) + delta),
                        old_listpack_bytes as usize - eleoff,
                    );

                    v.encode(p, encoded_size);
                }
                None => {
                    // Treat it as EOF
                    v.encode(p, encoded_size);

                    // Write EOF
                    *lp.offset((new_listpack_bytes as isize) - 1) = EOF;
                }
            }

            // Reduce allocation.
            lp = allocator.realloc(lp, new_listpack_bytes);
            if lp.is_null() {
                return None;
            }

            // Update "p" to it's pointer in the potentially new allocation.
            p = lp.offset(poff as isize);
        }

        // Update bytes
        set_total_bytes(lp, new_listpack_bytes as u32);

        Some((lp, p))
    }
}

///
#[inline]
pub fn replace_int<'a, A, I>(
    allocator: &'a A,
    mut lp: listpack,
    mut p: element,
    mut v: I,
) -> Option<(listpack, element)>
    where
        A: Allocator,
        I: Int {
    replace(allocator, lp, p, Value::Int(v.to_int64()))
}

///
#[inline]
pub fn replace_signed_int<'a, A, I>(
    allocator: &'a A,
    mut lp: listpack,
    mut p: element,
    mut v: I,
) -> Option<(listpack, element)>
    where
        A: Allocator,
        I: Int {
    replace(allocator, lp, p, Value::Int(zigzag(v.to_int64())))
}

///
#[inline]
pub fn replace_string<A, S>(
    allocator: &A,
    mut lp: listpack,
    mut p: element,
    mut v: S,
) -> Option<(listpack, element)>
    where
        A: Allocator,
        S: Str {
    replace(allocator, lp, p, v.as_value())
}


//===----------------------------------------------------------------------===//
// Append
//===----------------------------------------------------------------------===//

///
#[inline]
pub fn append<A>(
    allocator: &A,
    mut lp: listpack,
    v: Value,
) -> Option<listpack>
    where A: Allocator {
    unsafe {
        let encoded_size = v.size_for_write();

        // Calculate the old and new sizes.
        let old_listpack_bytes = get_total_bytes(lp);
        let new_listpack_bytes = old_listpack_bytes + encoded_size;
        if new_listpack_bytes > u32::max_value() {
            return None;
        }

        // realloc to make room
        lp = allocator.realloc(lp, new_listpack_bytes as usize);
        if lp.is_null() {
            return None;
        }

        // Locate EOF marker.
        let p = lp.offset(old_listpack_bytes as isize - 1);

        // Write value.
        // This overwrites the EOF byte at the end which will get added
        // immediately after this new value.
        v.encode(p, encoded_size);

        // Write EOF
        *lp.offset(new_listpack_bytes as isize - 1) = EOF;

        // Update header
        let num_elements = get_num_elements(lp);
        if num_elements != HDR_NUMELE_UNKNOWN {
            set_num_elements(lp, num_elements + 1);
        }
        set_total_bytes(lp, new_listpack_bytes);

        Some(lp)
    }
}

///
#[inline]
pub fn append_val<'a, 'b, A>(
    allocator: &'a A,
    mut lp: listpack,
    v: &'b Value,
) -> (&'b Value, Option<listpack>)
    where A: Allocator {
    unsafe {
        let encoded_size = v.size_for_write();

        // Calculate the old and new sizes.
        let old_listpack_bytes = get_total_bytes(lp);
        let new_listpack_bytes = old_listpack_bytes + encoded_size;
        if new_listpack_bytes > u32::max_value() {
            return (v, None);
        }

        // realloc to make room
        lp = allocator.realloc(lp, new_listpack_bytes as usize);
        if lp.is_null() {
            return (v, None);
        }

        // Locate EOF marker.
        let p = lp.offset(old_listpack_bytes as isize - 1);

        // Write value.
        // This overwrites the EOF byte at the end which will get added
        // immediately after this new value.
        v.encode(p, encoded_size);

        // Write EOF
        *lp.offset(new_listpack_bytes as isize - 1) = EOF;

        // Update header
        let num_elements = get_num_elements(lp);
        if num_elements != HDR_NUMELE_UNKNOWN {
            set_num_elements(lp, num_elements + 1);
        }
        set_total_bytes(lp, new_listpack_bytes);

        (v, Some(lp))
    }
}

#[repr(packed)]
/// The pointer to the new listpack and the element and length of the
/// bytes that were written.
pub struct WriteResult(Option<listpack>, *mut u8, u32);

#[inline]
/// Reverts a listpack back to a marker without resizing the actual allocation.
pub fn revert(mut lp: listpack, size: u32, count: u16) {
    set_total_bytes(lp, size);
    set_num_elements(lp, count);
    unsafe { *lp.offset((size - 1) as isize) = EOF; }
}

///
#[inline]
pub fn append_writes<A>(
    mut lp: listpack,
    lp_size: u32,
    layout_size: u32,
    v: &Value,
) -> WriteResult
    where A: Allocator {
    unsafe {
        let encoded_size = v.size_for_write();

        // Calculate the old and new sizes.
        let new_listpack_bytes = lp_size + encoded_size;
        if new_listpack_bytes > layout_size {
            if new_listpack_bytes > u32::max_value() {
                return WriteResult(None, ptr::null_mut(), 0);
            }
            // realloc to make room
            lp = ::alloc::realloc(lp, new_listpack_bytes as usize);
            if lp.is_null() {
                return WriteResult(None, ptr::null_mut(), 0);
            }
        }

        // Locate EOF marker.
        let p = lp.offset(lp_size as isize - 1);

        // Write value.
        // This overwrites the EOF byte at the end which will get added
        // immediately after this new value.
        v.encode(p, encoded_size);

        // Write EOF
        *lp.offset(new_listpack_bytes as isize - 1) = EOF;

        // Update header
        let num_elements = get_num_elements(lp);
        if num_elements != HDR_NUMELE_UNKNOWN {
            set_num_elements(lp, num_elements + 1);
        }
        set_total_bytes(lp, new_listpack_bytes);

        // Note this does not include the EOF byte.
        WriteResult(Some(lp), p, encoded_size)
    }
}

///
#[inline]
pub fn append_write<A>(
    allocator: &A,
    mut lp: listpack,
    lp_size: u32,
    layout_size: u32,
    v: &Value,
) -> WriteResult
    where A: Allocator {
    unsafe {
        let encoded_size = v.size_for_write();

        // Calculate the old and new sizes.
        let new_listpack_bytes = lp_size + encoded_size;
        if new_listpack_bytes > layout_size {
            if new_listpack_bytes > u32::max_value() {
                return WriteResult(None, ptr::null_mut(), 0);
            }
            // realloc to make room
            lp = allocator.realloc(lp, new_listpack_bytes as usize);
            if lp.is_null() {
                return WriteResult(None, ptr::null_mut(), 0);
            }
        }

        // Locate EOF marker.
        let p = lp.offset(lp_size as isize - 1);

        // Write value.
        // This overwrites the EOF byte at the end which will get added
        // immediately after this new value.
        v.encode(p, encoded_size);

        // Write EOF
        *lp.offset(new_listpack_bytes as isize - 1) = EOF;

        // Update header
        let num_elements = get_num_elements(lp);
        if num_elements != HDR_NUMELE_UNKNOWN {
            set_num_elements(lp, num_elements + 1);
        }
        set_total_bytes(lp, new_listpack_bytes);

        // Note this does not include the EOF byte.
        WriteResult(Some(lp), p, encoded_size)
    }
}

///
#[inline(always)]
pub fn append_int<'a, A, I>(
    allocator: &'a A,
    mut lp: listpack,
    v: I,
) -> Option<listpack>
    where A: Allocator, I: Int {
    append(allocator, lp, Value::Int(v.to_int64()))
}

/// Zigzag encodes the integer for a potentially smaller encoded value.
/// Using this encoding allows small negatives like -1 or -20 to be
/// encoded using a single byte.
#[inline(always)]
pub fn append_signed_int<'a, A, I>(
    allocator: &'a A,
    mut lp: listpack,
    v: I,
) -> Option<listpack>
    where A: Allocator, I: Int {
    append(allocator, lp, Value::Int(zigzag(v.to_int64())))
}

///
#[inline(always)]
pub fn append_string<'a, A, S>(
    allocator: &'a A,
    mut lp: listpack,
    mut v: S,
) -> Option<listpack>
    where A: Allocator, S: Str {
    append(allocator, lp, v.as_value())
}


//===----------------------------------------------------------------------===//
// Delete
//===----------------------------------------------------------------------===//

///
pub fn delete<'a, A>(
    allocator: &'a A,
    mut lp: listpack,
    p: element,
) -> Option<(listpack, element)>
    where A: Allocator {
    unsafe {
        let encoded_size = get_encoded_size(p);
        let backlen_size = backlen_size(encoded_size as u64);
        let entry_size = (encoded_size + backlen_size) as usize;

        let newp = match next(lp, p) {
            Some(ele) => ele,
            None => {
                let old_listpack_bytes = get_total_bytes(lp);
                if old_listpack_bytes == (HDR_USIZE as u32) + 1 {
                    return None;
                }

                let new_listpack_bytes = old_listpack_bytes - encoded_size - backlen_size;
                lp = allocator.realloc(lp, new_listpack_bytes as usize);
                *lp.offset(new_listpack_bytes as isize - 1) = EOF;

                return Some((
                    lp,
                    last(lp).unwrap_or(std::ptr::null_mut())
                ));
            }
        };

        let p_ptr = p as usize;
        let lp_ptr = lp as usize;
        if p_ptr < lp_ptr {
            return None;
        }

        let poff = p_ptr - lp_ptr;
        let old_listpack_bytes = get_total_bytes(lp);
        let new_listpack_bytes = old_listpack_bytes - (entry_size as u32);

        std::ptr::copy(
            lp.offset((poff as isize) + (entry_size as isize)),
            lp.offset(poff as isize),
            (old_listpack_bytes as usize) - (poff + entry_size));

        // Resize allocation down.
        lp = allocator.realloc(lp, new_listpack_bytes as usize);

        // Update header
        let num_elements = get_num_elements(lp);
        if num_elements != HDR_NUMELE_UNKNOWN {
            set_num_elements(lp, num_elements - 1);
        }
        set_total_bytes(lp, new_listpack_bytes);

        Some((lp, p))
    }
}

//===----------------------------------------------------------------------===//
// Iterate
//===----------------------------------------------------------------------===//

pub fn iter<F>(lp: listpack, f: F) where F: Fn(element, Value) -> bool {
    match first(lp) {
        None => return,
        Some(mut ele) => {
            if !f(ele, get(ele)) {
                return;
            }
            while let Some(p) = next(lp, ele) {
                if !f(p, get(p)) {
                    return;
                }
                ele = p;
            }
        }
    }
}

pub fn iter_index<F>(lp: listpack, f: F) where F: Fn(usize, element, Value) -> bool {
    let mut index = 0;
    match first(lp) {
        None => return,
        Some(mut ele) => {
            if !f(index, ele, get(ele)) {
                return;
            }
            index = index + 1;
            while let Some(p) = next(lp, ele) {
                if !f(index, p, get(p)) {
                    return;
                }
                ele = p;
                index = index + 1;
            }
        }
    }
}

pub fn iter_rev<F>(lp: listpack, f: F) where F: Fn(element, Value) -> bool {
    match last(lp) {
        None => return,
        Some(mut ele) => {
            if !f(ele, get(ele)) {
                return;
            }
            while let Some(p) = prev(lp, ele) {
                if !f(p, get(p)) {
                    return;
                }
                ele = p;
            }
        }
    }
}

//===----------------------------------------------------------------------===//
// Int - Conversions
//===----------------------------------------------------------------------===//

macro_rules! impl_into_value_int {
    ($t:ty) => {
        impl Into<Value> for $t {
            #[inline]
            fn into(self) -> Value {
                Value::Int(self as i64)
            }
        }

        impl<'a> Into<Value> for &'a $t {
            #[inline]
            fn into(self) -> Value {
                Value::Int(*self as i64)
            }
        }
    }
}

macro_rules! impl_from_value_uint {
    ($t:ty) => {
        impl From<Value> for $t {
            fn from(v: Value) -> Self {
                match v {
                    Value::Int(i) => i as Self,
                    Value::String(ptr, len) => unsafe {
                        match len {
                            1 => *ptr as u8 as Self,
                            2 => u16::from_le(*(ptr as *mut [u8; size_of::<u16>()] as *mut u16)) as Self,
                            4 => u32::from_le(*(ptr as *mut [u8; size_of::<u32>()] as *mut u32)) as Self,
                            8 => u64::from_le(*(ptr as *mut [u8; size_of::<u64>()] as *mut u64)) as Self,
                            16 => Self::from_le(*(ptr as *mut [u8; size_of::<u128>()] as *mut Self)),
                            _ => Self::default()
                        }
                    }
                }
            }
        }
    }
}

macro_rules! impl_from_value_int {
    ($t:ty) => {
        impl From<Value> for $t {
            fn from(v: Value) -> Self {
                match v {
                    Value::Int(i) => i as Self,
                    Value::String(ptr, len) => unsafe {
                        match len {
                            1 => *ptr as i8 as Self,
                            2 => i16::from_le(*(ptr as *mut [u8; size_of::<i16>()] as *mut i16)) as Self,
                            4 => i32::from_le(*(ptr as *mut [u8; size_of::<i32>()] as *mut i32)) as Self,
                            8 => i64::from_le(*(ptr as *mut [u8; size_of::<i64>()] as *mut i64)) as Self,
                            16 => Self::from_le(*(ptr as *mut [u8; size_of::<i128>()] as *mut Self)),
                            _ => Self::default()
                        }
                    }
                }
            }
        }
    }
}

///
impl Into<Value> for bool {
    #[inline]
    fn into(self) -> Value {
        if self {
            Value::Int(1)
        } else {
            Value::Int(0)
        }
    }
}

impl From<Value> for bool {
    #[inline]
    fn from(v: Value) -> Self {
        match v {
            Value::Int(i) => i > 0,
            Value::String(ptr, len) => unsafe {
                if len == 0 || ptr.is_null() {
                    false
                } else {
                    *ptr > 0
                }
            }
        }
    }
}

impl_into_value_int!(u8);
impl_from_value_uint!(u8);
impl_into_value_int!(i8);
impl_from_value_int!(i8);
impl_into_value_int!(u16);
impl_from_value_uint!(u16);
impl_into_value_int!(i16);
impl_from_value_int!(i16);
impl_into_value_int!(u32);
impl_from_value_uint!(u32);
impl_into_value_int!(i32);
impl_from_value_int!(i32);
impl_into_value_int!(u64);
impl_from_value_uint!(u64);
impl_into_value_int!(i64);
impl_from_value_int!(i64);
impl_into_value_int!(usize);
impl_from_value_uint!(usize);
impl_into_value_int!(isize);
impl_from_value_int!(isize);


//===----------------------------------------------------------------------===//
// Float - Conversions
//===----------------------------------------------------------------------===//

impl Into<Value> for f32 {
    #[inline]
    fn into(self) -> Value {
        Value::Int(self.to_bits() as i64)
    }
}

impl From<Value> for f32 {
    #[inline]
    fn from(v: Value) -> Self {
        match &v {
            &Value::Int(i) => Self::from_bits(i as u32),
            &Value::String(ptr, len) => unsafe {
                match len {
                    1 => *ptr as u8 as Self,
                    2 => f32::from_bits(
                        u16::from_le(
                            *(ptr as *mut [u8; size_of::<u16>()] as *mut u16)
                        ) as u32
                    ),
                    4 => f32::from_bits(
                        u32::from_le(
                            *(ptr as *mut [u8; size_of::<Self>()] as *mut u32)
                        )
                    ),
                    8 => f64::from_bits(
                        u64::from_le(
                            *(ptr as *mut [u8; size_of::<Self>()] as *mut u64)
                        )
                    ) as f32,
                    _ => Self::default()
                }
            }
        }
    }
}

impl Into<Value> for f64 {
    #[inline]
    fn into(self) -> Value {
        Value::Int(self.to_bits() as i64)
    }
}

impl From<Value> for f64 {
    #[inline]
    fn from(v: Value) -> Self {
        match &v {
            &Value::Int(i) => Self::from_bits(i as u64),
            &Value::String(ptr, len) => unsafe {
                match len {
                    1 => *ptr as u8 as Self,
                    2 => f64::from_bits(
                        u16::from_le(
                            *(ptr as *mut [u8; size_of::<u16>()] as *mut u16)
                        ) as u64
                    ),
                    4 => f32::from_bits(
                        u32::from_le(
                            *(ptr as *mut [u8; size_of::<Self>()] as *mut u32)
                        )
                    ) as f64,
                    8 => f64::from_bits(
                        u64::from_le(
                            *(ptr as *mut [u8; size_of::<Self>()] as *mut u64)
                        )
                    ),
                    _ => Self::default()
                }
            }
        }
    }
}


impl_from_value_int!(i128);
impl_from_value_uint!(u128);


//impl Into<Value> for u128 {
//    #[inline]
//    fn into(self) -> Value {
//        Value::String(
//            &self as *const _ as *const u8,
//            size_of::<u128>() as u32,
//        )
//    }
//}
//
//impl Into<Value> for i128 {
//    #[inline]
//    fn into(self) -> Value {
//        Value::String(
//            &self as *const _ as *const u8,
//            std::mem::size_of::<i128>() as u32,
//        )
//    }
//}


macro_rules! impl_into_value_as_ptr {
    ($t:ty) => {
        impl<'a> Into<Value> for $t {
            #[inline]
            fn into(self) -> Value {
                Value::String(self.as_ptr(), self.len() as u32)
            }
        }
    }
}

impl_into_value_as_ptr!(&'a str);
impl_into_value_as_ptr!(&'a [u8]);
impl_into_value_as_ptr!(&'a Vec<u8>);
impl_into_value_as_ptr!(Vec<u8>);
impl_into_value_as_ptr!(String);
impl_into_value_as_ptr!(&'a String);

pub trait Int: Sync + Send + Sized + Clone + Default + std::fmt::Debug {
    fn to_int64(self) -> i64;
}

impl Int for bool {
    #[inline]
    fn to_int64(self) -> i64 {
        if self {
            1
        } else {
            0
        }
    }
}

macro_rules! impl_int {
    ($t:ty) => {
        impl Int for $t {
            #[inline]
            fn to_int64(self) -> i64 {
                self as i64
            }
        }
    }
}


impl_int!(isize);
impl_int!(usize);
impl_int!(i16);
impl_int!(u16);
impl_int!(i32);
impl_int!(u32);
impl_int!(i64);
impl_int!(u64);
impl_int!(i128);
impl_int!(u128);

/// String encoding transmutation helper trait.
pub trait Str: Sync + Send + Sized + Clone + Default + std::fmt::Debug {
    #[inline]
    fn as_value(&mut self) -> Value;
}

macro_rules! impl_str_float {
    ($t:ty) => {
        impl Str for $t {
            #[inline]
            fn as_value(&mut self) -> Value {
                unsafe {
                    *self = std::mem::transmute(self.to_bits().to_le());
                }
                Value::String(
                    self as *const _ as *const u8,
                    std::mem::size_of::<Self>() as u32,
                )
            }
        }
    }
}

impl_str_float!(f32);
impl_str_float!(f64);

macro_rules! impl_str_transmute_sized {
    ($t:ty) => {
        impl Str for $t {
            #[inline]
            fn as_value(&mut self) -> Value {
                Value::String(
                    self as *const _ as *const u8,
                    std::mem::size_of::<Self>() as u32,
                )
            }
        }
    }
}


impl_str_transmute_sized!(i8);
impl_str_transmute_sized!(u8);

macro_rules! impl_str_int {
    ($t:ty) => {
        impl Str for $t {
            #[inline]
            fn as_value(&mut self) -> Value {
                unsafe { *self = self.to_le(); }
                Value::String(
                    self as *const _ as *const u8,
                    std::mem::size_of::<Self>() as u32
                )
            }
        }
    }
}
impl_str_int!(isize);
impl_str_int!(usize);
impl_str_int!(i16);
impl_str_int!(u16);
impl_str_int!(i32);
impl_str_int!(u32);
impl_str_int!(i64);
impl_str_int!(u64);
impl_str_int!(i128);
impl_str_int!(u128);


macro_rules! impl_str_as_ptr {
    ($t:ty) => {
        impl<'a> Str for $t {
            #[inline]
            fn as_value(&mut self) -> Value {
                Value::String(
                    self.as_ptr(),
                    self.len() as u32,
                )
            }
        }
    }
}
impl_str_as_ptr!(Vec<u8>);
impl_str_as_ptr!(&'a [u8]);
impl_str_as_ptr!(&'a str);



#[cfg(test)]
mod tests {
    use redis::listpack::*;
    use alloc::*;
    use std;

    #[test]
    fn test_it() {
        unsafe {
            let mut lp = new(ALLOCATOR);

            lp = append(ALLOCATOR, lp, Value::Int(20)).expect("");
            lp = append(ALLOCATOR, lp, Value::Int(21)).expect("");
            lp = append(ALLOCATOR, lp, Value::Int(22)).expect("");

//            lp = append(ALLOCATOR, lp, 20f64.into()).expect("");

            let s = "hello";
            lp = append(ALLOCATOR, lp, Value::String(s.as_ptr(), s.len() as u32)).expect("");

            let mut p = first(lp as *mut u8).expect("No First!");

            let (mut lp, mut p) = insert(ALLOCATOR, lp, Value::Int(1), Placement::After, p).unwrap();
            let (mut lp, mut p) = insert(ALLOCATOR, lp, Value::Int(2), Placement::After, p).unwrap();
            let (mut lp, mut p) = insert(ALLOCATOR, lp, Value::Int(3), Placement::After, last(lp).unwrap()).unwrap();

//            let Some((mut lp, mut p)) = delete(ALLOCATOR, lp, p);
//            let Some((mut lp, mut p)) = delete(ALLOCATOR, lp, next(lp, first(lp).unwrap()).unwrap());

            iter(lp, |ele, value| {
                match value {
                    Value::Int(int) => {
                        println!("Int:    {}", int);
                    }
                    Value::String(string, slen) => {
                        println!("String: {}",
                                 std::str::from_utf8_unchecked(std::slice::from_raw_parts(string, slen as usize))
                        );
                    }
                }
                true
            });

            println!("Bytes:  {}", get_total_bytes(lp));
            println!("Length: {}", get_num_elements(lp));
        }
    }
}