#![allow(non_upper_case_globals)]
extern crate actix;
#[macro_use]
extern crate bitflags;
extern crate dlopen;
#[macro_use]
extern crate dlopen_derive;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate time;
extern crate nix;

//extern crate libloading as lib;
//#[macro_use]
//extern crate cpp;

use actix::{Actor, Addr, Arbiter, Context, msgs, System};
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
pub mod bg;
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
//static mut REDIS: *const redis::Redis = std::ptr::null();


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

//static mut APP: Option<&'static mut App> = None;
//static mut APP: &'static App = &App { redis: redis::Redis { ctx: std::ptr::null_mut() }, ctx: std::ptr::null_mut() };
static mut REDIS: &'static redis::Redis = &redis::Redis { ctx: std::ptr::null_mut() };
static mut CTX: *mut api::RedisModuleCtx = std::ptr::null_mut();
static mut COMMANDS: &Commands = &Commands {
    list: &mut []
};
//const BG: bg::Bg = bg::create();

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

    unsafe {
        CTX = ctx;

//        APP = Box::leak(Box::new(App {
//            redis: redis::Redis { ctx },
//            ctx,
//        }));

        // Load redis-server symbols
        RED_SYM = &REDIS_API.clone();
        REDIS = Box::leak(Box::new(redis::Redis { ctx }));
    }



    let redis = redis::Redis { ctx };

    let timer_id = redis.run(move || {
        println!("timer tick");
    });

    /**********************************************************************/
    // Intercept all commands
    /**********************************************************************/

    if api::subscribe_to_keyspace_events(
        ctx,
        api::NotifyFlags::ALL,
        Some(sliced_on_keyspace_event)) == api::Status::Err {
        return api::Status::Err;
    }

    let listpack = redis::listpack::Listpack::new();
    let len = listpack.len();

    /**********************************************************************/
    // Load DataTypes
    /**********************************************************************/

    // Create native Redis types
    if types::load(ctx) == api::Status::Err {
        return api::Status::Err;
    }

    /**********************************************************************/
    // Load Commands
    /**********************************************************************/

    // Load throttle commands
    if cmd::throttle::load(ctx, argv, argc) == api::Status::Err {
        return api::Status::Err;
    }

    // Load stream commands
    if cmd::stream::load(ctx, argv, argc) == api::Status::Err {
        return api::Status::Err;
    }

//    let red = redis::Redis { ctx };
    std::thread::spawn( || {
        actix::System::run( || {
            let addr = bg::Bg { redis: redis::Redis{ ctx: unsafe { CTX } }}.start();

            addr.do_send(bg::stream::Load);

            println!("started background system");
        });
    });

    println!("module loaded");
    api::Status::Ok
}

pub struct Types {
    pub list: &'static mut [&'static redis::DataType],
}

pub struct Commands {
    pub list: &'static mut [&'static redis::Command],
}


#[derive(WrapperApi, Clone, Copy, Debug)]
#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub struct RedisApi {
//    #[allow(non_snake_case)]
//    RM_SubscribeToKeyspaceEvents: extern "C" fn(ctx: *mut api::RedisModuleCtx,
//                                                types: libc::c_int,
//                                                callback: Option<api::RedisModuleNotificationFunc>) -> libc::c_int,
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