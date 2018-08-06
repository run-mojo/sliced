use libc;
use time;

use error::SlicedError;
use redis::{Command, Redis};
use redis::api;

///
pub fn load(
    ctx: *mut api::RedisModuleCtx,
    _argv: *mut *mut api::RedisModuleString,
    _argc: libc::c_int,
) -> api::Status {
    let command = VersionCommand {};
    if api::create_command(
        ctx,
        format!("{}\0", command.name()).as_ptr(),
        Some(Sliced_Version_RedisCommand),
        format!("{}\0", command.str_flags()).as_ptr(),
        0,
        0,
        0,
    ) == api::Status::Err {
        return api::Status::Err;
    }
    return api::Status::Ok;
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn Sliced_Version_RedisCommand(
    ctx: *mut api::RedisModuleCtx,
    argv: *mut *mut api::RedisModuleString,
    argc: libc::c_int,
) -> api::Status {
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
