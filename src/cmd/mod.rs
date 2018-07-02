extern crate libc;
extern crate time;

pub mod cmd;
pub mod throttle;
pub mod stream;

use error::{CellError};

pub fn parse_i64(arg: &str) -> Result<i64, CellError> {
    arg.parse::<i64>()
        .map_err(|_| error!("Couldn't parse as integer: {}", arg))
}
