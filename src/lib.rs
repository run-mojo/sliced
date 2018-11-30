// `const_fn` is needed for `spin::Once`.
#![feature(async_await, await_macro, pin, arbitrary_self_types, futures_api)]
#![feature(allocator_api)]

//#![cfg_attr(feature = "no_std", feature(const_fn))]

//extern crate dlopen;
//#[macro_use]
//extern crate dlopen_derive;

#[macro_use]
extern crate bitflags;
#[cfg(target_os = "windows")]
extern crate kernel32;
#[macro_use]
extern crate lazy_static;
#[cfg(unix)]
extern crate libc;
#[macro_use]
extern crate smallvec;
//#[cfg(feature = "no_std")]
extern crate spin;
extern crate time;
#[cfg(windows)]
extern crate winapi;

//extern crate hyper;

//use dlopen::wrapper::{Container, WrapperApi};
use self::redis::redmod;

#[macro_use]
mod macros;

/// Modules
pub mod mmap;
pub mod cell;
pub mod cmd;
pub mod error;
pub mod redis;
pub mod types;
pub mod alloc;
pub mod stream;

#[global_allocator]
/// Use RedisModule_Alloc/Realloc/Free for heap memory management.
/// It's essential that RedisModule_OnLoad be created in C which
/// makes the RedisModule_Init call at which point we can set the
/// global static functions before Rust gets initialized.
static GLOBAL: alloc::RedisAllocator = alloc::RedisAllocator;

#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn sliced_on_keyspace_event(
    ctx: *mut redmod::RedisModuleCtx,
    rtype: libc::c_int,
    event: *mut u8,
    key: *mut redmod::RedisModuleString,
) -> libc::c_int {
    println!("keyspace_event: {}", rtype);
    return 1;
}

/// Redis Module entry point
#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn RedisModule_DoLoad(
    ctx: *mut redmod::RedisModuleCtx,
    argv: *mut *mut redmod::RedisModuleString,
    argc: libc::c_int,
) -> redmod::Status {
    unsafe {
        // Bind SDS allocator.
        redis::sds::set_allocator(
            redmod::redis_malloc,
            redmod::redis_realloc,
            redmod::redis_free,
        );

        // Bind allocator to the RedisModule allocator.
        redis::rax::set_allocator(
            redmod::redis_malloc,
            redmod::redis_realloc,
            redmod::redis_free,
        );
    }

    /**********************************************************************/
    // Intercept all commands
    /**********************************************************************/

    if redmod::subscribe_to_keyspace_events(
        ctx,
        redmod::NotifyFlags::ALL,
        Some(sliced_on_keyspace_event)) == redmod::Status::Err {
        return redmod::Status::Err;
    }

    /**********************************************************************/
    // Load DataTypes
    /**********************************************************************/

    // Create native Redis types
    if types::load(ctx) == redmod::Status::Err {
        return redmod::Status::Err;
    }

    // Stream Data Type
    if stream::data_type::load(ctx) == redmod::Status::Err {
        return redmod::Status::Err;
    }

    /**********************************************************************/
    // Load Commands
    /**********************************************************************/

    // Load version commands
    if cmd::version::load(ctx, argv, argc) == redmod::Status::Err {
        return redmod::Status::Err;
    }

    // Load throttle commands
    if cmd::throttle::load(ctx, argv, argc) == redmod::Status::Err {
        return redmod::Status::Err;
    }

    // Load stream commands
    if cmd::stream::load(ctx, argv, argc) == redmod::Status::Err {
        return redmod::Status::Err;
    }

    println!("slice/d module loaded... Happy slicing!");
    redmod::Status::Ok
}


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
//
//static mut RED_SYM: *const RedisApi = std::ptr::null();
//
//static mut CTX: *mut api::RedisModuleCtx = std::ptr::null_mut();
//static mut COMMANDS: &Commands = &Commands {
//    list: &mut []
//};

//#[derive(WrapperApi, Clone, Copy, Debug)]
//#[allow(non_snake_case)]
//#[allow(unused_variables)]
//#[no_mangle]
///// Use this to hook into non-exposed Redis APIs. This is a bit of risky
///// business, but opens up some doors.
//pub struct RedisApi {
////    #[allow(non_snake_case)]
////    RM_SubscribeToKeyspaceEvents: extern "C" fn(ctx: *mut api::RedisModuleCtx,
////                                                types: libc::c_int,
////                                                callback: Option<api::RedisModuleNotificationFunc>) -> libc::c_int,
//}
