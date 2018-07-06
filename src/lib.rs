#![allow(non_upper_case_globals)]
#[macro_use]
extern crate bitflags;
//#[macro_use]
//extern crate const_cstr;
extern crate dlopen;
#[macro_use]
extern crate dlopen_derive;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate time;

//extern crate libloading as lib;
//#[macro_use]
//extern crate cpp;

//use dlopen::raw::Library;
use dlopen::wrapper::{Container, WrapperApi};
//use libc::{c_int};
// Imports
//use std::alloc::{GlobalAlloc, Layout};
//use std::ffi::CStr;
use redis::api;

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

//extern "C" {
//    fn zmalloc()
//}


lazy_static! {
    static ref REDIS_API: Container<RedisApi> = {
        (match std::env::var("REDIS_PATH") {
            Ok(path) => {
            println!("{}", path);
                Some(unsafe { Container::load(path) }.expect("Could not open library"))
            },
            Err(_) => {
                match std::env::current_exe() {
                    Ok(exe_path) => {
                    println!("{}", exe_path.to_str().unwrap());;
                        Some(unsafe { Container::load(exe_path.to_str().unwrap()) }.expect("Could not open library"))
                    },
                    Err(e) => None
                }
            }
        }).unwrap()
    };
}

static mut RED_SYM: *const RedisApi = std::ptr::null();
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

//static mut GLOBAL: Option<Global> = None;

#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn sliced_on_keyspace_event(
    ctx: *mut api::RedisModuleCtx,
    rtype: libc::c_int,
    event: *mut u8,
    key: *mut api::RedisModuleString,
) -> libc::c_int {
    println!("keyspace_event");
    return 1;
}

const CLO: fn() = || {};

/// Redis Module entry point
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

    let GLOBAL = Global {
        redis: redis::Redis { ctx },
        ctx,
    };

    unsafe {
        // Load redis-server symbols
        RED_SYM = &REDIS_API.clone();
        REDIS = &redis::Redis { ctx };
    }

    let redis = redis::Redis { ctx };

    let timer_id = redis.run(move || {
        println!("timer tick");
    });


    if api::subscribe_to_keyspace_events(
        ctx,
        api::NotifyFlags::ALL,
        Some(sliced_on_keyspace_event)) == api::Status::Err {
        return api::Status::Err;
    }

    let listpack = redis::listpack::ListPack::new();
    let len = listpack.length();


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

//const GLOBAL: Global<'static> = Global {
//    redis: &mut redis::Redis { ctx: std::ptr::null_mut() },
//    ctx: std::ptr::null_mut(),
////    timers: (),
//};

//#[derive(Clone)]
pub struct Global {
    pub redis: redis::Redis,
    pub ctx: *mut api::RedisModuleCtx,
//    pub timers: std::collections::HashMap<redis::TimerID, redis::TimerHandle<'a>>,

    // data_types
    // commands
}


impl Global {
//    pub fn create_timer(&self, millis: i64, callback: fn() -> i32) {
//        let cb = Box::new(Box::new(callback));
//
//        api::create_timer(self.ctx, millis, Some(sliced_timer_callback), unsafe { Box::into_raw(cb) as *mut libc::c_void });
//    }
}


#[derive(WrapperApi, Clone, Copy, Debug)]
#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub struct RedisApi {
    RM_SubscribeToKeyspaceEvents: extern "C" fn(ctx: *mut api::RedisModuleCtx,
                                                types: libc::c_int,
                                                callback: Option<api::RedisModuleNotificationFunc>) -> libc::c_int,
}


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