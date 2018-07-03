#[macro_use]
extern crate bitflags;
extern crate libc;
extern crate time;

#[macro_use]
mod macros;

/// Modules
pub mod cell;
pub mod cmd;
pub mod error;
pub mod redis;
pub mod types;

// Imports
use redis::api;

/// Module name and version
const MODULE_NAME: &'static str = "slice/d";
const MODULE_VERSION: libc::c_int = 1;

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn RedisModule_OnLoad(
    ctx: *mut api::RedisModuleCtx,
    argv: *mut *mut api::RedisModuleString,
    argc: libc::c_int,
) -> api::Status {
    if api::init(
        ctx,
        format!("{}\0", MODULE_NAME).as_ptr(),
        MODULE_VERSION,
        api::REDISMODULE_APIVER_1,
    ) == api::Status::Err {
        return api::Status::Err;
    }

    // Create native Redis types
    if types::create_redis_types(ctx) == api::Status::Err {
        return api::Status::Err;
    }

    // Load throttle commands
    if cmd::throttle::load(ctx, argv, argc) == api::Status::Err {
        return api::Status::Err;
    }

    // Load stream commands
    if cmd::stream::load(ctx, argv, argc) == api::Status::Err {
        return api::Status::Err;
    }

    api::Status::Ok
}


// extern "C" {
//     pub fn SD_Alloc(size: libc::size_t) -> *mut libc::c_void;
//     pub fn SD_Realloc(p: *mut libc::c_void, size: libc::size_t) -> *mut libc::c_void;
//     pub fn SD_Free(p: *mut libc::c_void);
// }

// /// Redis memory allocator (jemalloc)
// struct RedisAllocator;

// /// Use the Redis's jemalloc. This taps into the memory management currently in Redis.
// /// Memory allocated in slice/d will show up in the "MEMORY" command.
// unsafe impl GlobalAlloc for RedisAllocator {
//     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
//         SD_Alloc(layout.size() as libc::size_t) as *mut u8
//     }

//     unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
//         SD_Free(ptr as *mut libc::c_void)
//     }

//     unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
//         SD_Realloc(ptr as *mut libc::c_void, new_size as libc::size_t) as *mut u8
//     }
// }

// /// Wire up Rust's global allocator
// #[global_allocator]
// static GLOBAL: RedisAllocator = RedisAllocator;