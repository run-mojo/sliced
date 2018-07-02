#[macro_use]
extern crate bitflags;
extern crate libc;
extern crate time;

#[macro_use]
mod macros;

/// Modules
pub mod cell;
pub mod error;
pub mod types;
pub mod cmd;
pub mod redis;

// Imports
use error::CellError;
use redis::raw;
use std::alloc::{Layout, GlobalAlloc};

/// Redis memory allocator (jemalloc)
struct RedisAllocator;

/// Use the Redis's jemalloc. This taps into the memory management currently in Redis.
/// Memory allocated in slice/d will show up in the "MEMORY" command.
unsafe impl GlobalAlloc for RedisAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        redis::raw::zmalloc(layout.size() as libc::size_t) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        redis::raw::zfree(ptr as *mut libc::c_void)
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        redis::raw::zrealloc(ptr as *mut libc::c_void, new_size as libc::size_t) as *mut u8
    }
}

/// Wire up Rust's global allocator
#[global_allocator]
static GLOBAL: RedisAllocator = RedisAllocator;

/// Module name and version
const MODULE_NAME: &'static str = "slice/d";
const MODULE_VERSION: libc::c_int = 1;

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn RedisModule_OnLoad(
    ctx: *mut raw::RedisModuleCtx,
    argv: *mut *mut raw::RedisModuleString,
    argc: libc::c_int,
) -> raw::Status {
    if raw::init(
        ctx,
        format!("{}\0", MODULE_NAME).as_ptr(),
        MODULE_VERSION,
        raw::REDISMODULE_APIVER_1,
    ) == raw::Status::Err {
        return raw::Status::Err;
    }

    // Create native Redis types
    if types::create_redis_types(ctx) == raw::Status::Err {
        return raw::Status::Err;
    }

    // Load throttle commands
    if cmd::throttle::load(ctx, argv, argc) == raw::Status::Err {
        return raw::Status::Err;
    }

    // Load stream commands
    if cmd::stream::load(ctx, argv, argc) == raw::Status::Err {
        return raw::Status::Err;
    }

    raw::Status::Ok
}

fn parse_i64(arg: &str) -> Result<i64, CellError> {
    arg.parse::<i64>()
        .map_err(|_| error!("Couldn't parse as integer: {}", arg))
}