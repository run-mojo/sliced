// `const_fn` is needed for `spin::Once`.
#![feature(async_await, await_macro, pin, arbitrary_self_types, futures_api)]

// Rc clone
#![feature(optin_builtin_traits)]
#![feature(box_into_raw_non_null)]
#![feature(core_intrinsics)]
#![feature(dropck_eyepatch)]
#![feature(allocator_api)]
#![feature(ptr_internals)]
#![feature(specialization)]
#![feature(coerce_unsized)]
#![feature(unsize)]
#![feature(lang_items)]
#![feature(unboxed_closures)]
#![feature(generator_trait)]
#![feature(exact_size_is_empty)]
#![feature(fn_traits)]
// End Rc clone

// raw_vec
#![feature(const_fn)]
#![feature(try_reserve)]

#![cfg_attr(feature = "no_std", feature(const_fn))]


//#[cfg(feature = "no_std")]
extern crate spin;
#[cfg(unix)]
extern crate libc;
#[cfg(unix)]
extern crate nix;
//#[cfg(not(feature = "no_std"))]
//extern crate std;

#[cfg(windows)]
extern crate winapi;
#[cfg(target_os = "windows")]
extern crate kernel32;

#[macro_use]
extern crate bitflags;
extern crate dlopen;
#[macro_use]
extern crate dlopen_derive;
#[macro_use]
extern crate lazy_static;
extern crate time;

use dlopen::wrapper::{Container, WrapperApi};
use self::redis::api;

#[macro_use]
mod macros;

/// Modules
pub mod page_size;
pub mod mmap;
pub mod bg;
pub mod cell;
pub mod cmd;
pub mod error;
pub mod redis;
pub mod types;
pub mod alloc;
pub mod stream;

/// Module name and version
const MODULE_NAME: &'static str = "slice/d";
const MODULE_VERSION: libc::c_int = 1;

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

static mut CTX: *mut api::RedisModuleCtx = std::ptr::null_mut();
static mut COMMANDS: &Commands = &Commands {
    list: &mut []
};


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

        // Load redis-server symbols
        RED_SYM = &REDIS_API.clone();

        // Bind allocator to the RedisModule allocator.
        redis::rax::set_allocator(
            api::redis_malloc,
            api::redis_realloc,
            api::redis_free,
        );
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
    if cmd::version::load(ctx, argv, argc) == api::Status::Err {
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

//    let red = redis::Redis { ctx };
    std::thread::spawn(|| {
//        actix::System::run(|| {
//            let addr = bg::Bg { redis: redis::Redis { ctx: unsafe { CTX } } }.start();
//
//            addr.do_send(bg::stream::Load);
//
//            println!("started background system");
//        });
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
/// Use this to hook into non-exposed Redis APIs. This is a bit of risky
/// business, but opens up some doors.
pub struct RedisApi {
//    #[allow(non_snake_case)]
//    RM_SubscribeToKeyspaceEvents: extern "C" fn(ctx: *mut api::RedisModuleCtx,
//                                                types: libc::c_int,
//                                                callback: Option<api::RedisModuleNotificationFunc>) -> libc::c_int,
}
