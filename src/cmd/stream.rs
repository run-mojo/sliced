extern crate libc;
extern crate time;

use crate::error::SlicedError;
use crate::redis::{Command, Redis};
use crate::redis::redmod;

///
pub fn load(
    ctx: *mut redmod::RedisModuleCtx,
    _argv: *mut *mut redmod::RedisModuleString,
    _argc: libc::c_int,
) -> redmod::Status {
    let command = AddCommand {};
    if redmod::create_command(
        ctx,
        format!("{}\0", command.name()).as_ptr(),
        Some(StreamAdd_RedisCommand),
        format!("{}\0", command.str_flags()).as_ptr(),
        0,
        0,
        0,
    ) == redmod::Status::Err {
        return redmod::Status::Err;
    }
    return redmod::Status::Ok;
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn StreamAdd_RedisCommand(
    ctx: *mut redmod::RedisModuleCtx,
    argv: *mut *mut redmod::RedisModuleString,
    argc: libc::c_int,
) -> redmod::Status {
    Command::harness(&AddCommand {}, ctx, argv, argc)
}

struct AddCommand {}

impl AddCommand {
    extern "C" fn timer_callback(_value: *mut libc::c_void) {
//        log_debug!(self, "timer_callback() called");
    }
}

impl Command for AddCommand {
    // Should return the name of the command to be registered.
    fn name(&self) -> &'static str {
        "mo.add"
    }

    fn run(&self, r: Redis, _: &[&str]) -> Result<(), SlicedError> {
        let _time_reply = r.call("GET", &["hi"]).unwrap();

        // Get throttle key
        r.reply_array(4)?;
        r.reply_integer(1)?;
        r.reply_integer(2)?;
        r.reply_integer(5)?;
        r.reply_integer(10)?;

        Ok(())
    }

    fn str_flags(&self) -> &'static str {
        "write"
    }
}
