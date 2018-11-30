use libc;
use time;

use crate::error::SlicedError;
use crate::redis::{Command, Redis};
use crate::redis::redmod;

///
pub fn load(
    ctx: *mut redmod::RedisModuleCtx,
    _argv: *mut *mut redmod::RedisModuleString,
    _argc: libc::c_int,
) -> redmod::Status {
    let command = VersionCommand {};
    if redmod::create_command(
        ctx,
        format!("{}\0", command.name()).as_ptr(),
        Some(Sliced_Version_RedisCommand),
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
pub extern "C" fn Sliced_Version_RedisCommand(
    ctx: *mut redmod::RedisModuleCtx,
    argv: *mut *mut redmod::RedisModuleString,
    argc: libc::c_int,
) -> redmod::Status {
    Command::harness(&VersionCommand {}, ctx, argv, argc)
}

struct VersionCommand {}

impl VersionCommand {
}

impl Command for VersionCommand {
    // Should return the name of the command to be registered.
    fn name(&self) -> &'static str {
        "mo.version"
    }

    fn run(&self, r: Redis, _: &[&str]) -> Result<(), SlicedError> {
        // Get throttle key
        r.reply_string("0.1.0")?;

        Ok(())
    }

    fn str_flags(&self) -> &'static str {
        "readonly"
    }
}
