extern crate libc;
extern crate time;

pub mod cmd;
pub mod throttle;
pub mod stream;
pub mod version;

use crate::error::{SlicedError};

pub fn parse_i64(arg: &str) -> Result<i64, SlicedError> {
    arg.parse::<i64>()
        .map_err(|_| error!("Couldn't parse as integer: {}", arg))
}
