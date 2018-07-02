extern crate libc;
extern crate time;

use error::{CellError};

use redis::{Command, Redis};
use redis::raw;

pub fn load(
    ctx: *mut raw::RedisModuleCtx,
    _argv: *mut *mut raw::RedisModuleString,
    _argc: libc::c_int,
) -> raw::Status {
    let command = StreamAddCommand {};
    if raw::create_command(
        ctx,
        format!("{}\0", command.name()).as_ptr(),
        Some(StreamAdd_RedisCommand),
        format!("{}\0", command.str_flags()).as_ptr(),
        0,
        0,
        0,
    ) == raw::Status::Err {
        return raw::Status::Err;
    }
    return raw::Status::Ok;
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn StreamAdd_RedisCommand(
    ctx: *mut raw::RedisModuleCtx,
    argv: *mut *mut raw::RedisModuleString,
    argc: libc::c_int,
) -> raw::Status {
    Command::harness(&StreamAddCommand {}, ctx, argv, argc)
}

struct StreamAddCommand {}

impl Command for StreamAddCommand {
    // Should return the name of the command to be registered.
    fn name(&self) -> &'static str {
        "mo.add"
    }

    fn run(&self, r: Redis, _: &[&str]) -> Result<(), CellError> {
        let _time_reply = r.call("GET", &["hi"]).unwrap();

        // Get throttle key
        r.reply_array(4)?;
        r.reply_integer(1)?;
        r.reply_integer(2)?;
        r.reply_integer(5)?;
        r.reply_integer(10)?;
//        r.reply_integer(raw::milliseconds());

        Ok(())
    }

    fn str_flags(&self) -> &'static str {
        "write"
    }
}