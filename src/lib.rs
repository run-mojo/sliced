#![allow(non_upper_case_globals)]
#[macro_use]
extern crate bitflags;
#[allow(non_snake_case)]
#[macro_use]
extern crate const_cstr;
extern crate dlopen;
#[macro_use]
extern crate dlopen_derive;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate libloading as lib;
extern crate time;

use dlopen::raw::Library;
use dlopen::wrapper::{Container, WrapperApi};
use lib::os::unix;
use libc::{c_char, c_int};
// Imports
use redis::api;
use std::alloc::{GlobalAlloc, Layout};
use std::ffi::CStr;

#[macro_use]
mod macros;


/// Modules
pub mod cell;
pub mod cmd;
pub mod error;
pub mod redis;
pub mod types;

/// Module name and version
const MODULE_NAME: &'static str = "slice/d";
const MODULE_VERSION: libc::c_int = 1;


//#[link(name = "jemalloc", kind = "static")]



//lazy_static! {
//    static ref REDIS_API: Container<RedisApi> = {
//        (match std::env::var("REDIS_PATH") {
//            Ok(path) => {
//            println!("{}", path);
//                Some(unsafe { Container::load(path) }.expect("Could not open library"))
//            },
//            Err(_) => {
//                match std::env::current_exe() {
//                    Ok(exe_path) => {
//                    println!("{}", exe_path.to_str().unwrap());;
//                        Some(unsafe { Container::load(exe_path.to_str().unwrap()) }.expect("Could not open library"))
//                    },
//                    Err(e) => None
//                }
//            }
//        }).unwrap()
//    };
//}

//static mut RED_SYM: *const RedisApi = std::ptr::null();
static mut REDIS: *const redis::Redis = std::ptr::null();


//fn load_redis_lib() -> Container<RedisApi> {
//    (match std::env::var("REDIS_PATH") {
//        Ok(path) => {
//            println!("{}", path);
//            Some(unsafe { Container::load(path) }.expect("Could not open library"))
//        }
//        Err(_) => {
//            match std::env::current_exe() {
//                Ok(exe_path) => {
//                    println!("{}", exe_path.to_str().unwrap());
//                    Some(unsafe { Container::load(exe_path) }.expect("Could not open library"))
//                }
//                Err(e) => None
//            }
//        }
//    }).unwrap()
//}
//
//fn bootstrap_redis() {
//    let redis_api = load_redis_lib();
//
//    unsafe {
//        // Load redis-server symbols
//        RED_SYM = &redis_api.clone();
//        REDIS = &redis::Redis { ctx: std::ptr::null_mut() };
//    }
//}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn RedisModule_OnLoad(
    ctx: *mut api::RedisModuleCtx,
    argv: *mut *mut api::RedisModuleString,
    argc: libc::c_int,
) -> api::Status {
    unsafe {
        // Load redis-server symbols
//        RED_SYM = &REDIS_API.clone();
        REDIS = &redis::Redis { ctx };
    }


    let listpack = redis::listpack::ListPack::new();
    let len = listpack.length();

    let listpack2 = redis::listpack::ListPack::new();
    let len2 = listpack2.length();

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


//#[derive(Clone)]
//pub struct Global {
//    pub redis: *const RedisApi,
//    pub ctx: *mut api::RedisModuleCtx,
//
//    // data_types
//    // commands
//}

//#[derive(WrapperApi, Clone, Copy, Debug)]
//#[allow(non_snake_case)]
//#[allow(unused_variables)]
//#[no_mangle]
//pub struct RedisApi {}


// extern "C" {
//     pub fn SD_Alloc(size: libc::size_t) -> *mut libc::c_void;
//     pub fn SD_Realloc(p: *mut libc::c_void, size: libc::size_t) -> *mut libc::c_void;
//     pub fn SD_Free(p: *mut libc::c_void);
// }

// Redis memory allocator (jemalloc)
// struct RedisAllocator;
//
//// /// Use the Redis's jemalloc. This taps into the memory management currently in Redis.
//// /// Memory allocated in slice/d will show up in the "MEMORY" command.
// unsafe impl GlobalAlloc for RedisAllocator {
//     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
//         (&REDIS_API).zmalloc(layout.size() as libc::size_t) as *mut u8
//     }
//
//     unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
//         (&REDIS_API).zfree(ptr as *mut libc::c_void)
//     }
//
//     unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
//         REDIS_API.zrealloc(ptr as *mut libc::c_void, new_size as libc::size_t) as *mut u8
//     }
// }
//
//// /// Wire up Rust's global allocator
// #[global_allocator]
// static GLOBAL: RedisAllocator = RedisAllocator;