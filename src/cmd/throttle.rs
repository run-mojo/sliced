extern crate libc;
extern crate time;

use super::super::error::{SlicedError};

use redis::{Command, Redis};
use redis::api;

use cell;
use cell::store;

use cmd;

pub fn load(
    ctx: *mut api::RedisModuleCtx,
    _argv: *mut *mut api::RedisModuleString,
    _argc: libc::c_int,
) -> api::Status {
    let command = ThrottleCommand {};
    if api::create_command(
        ctx,
        format!("{}\0", command.name()).as_ptr(),
        Some(Throttle_RedisCommand),
        format!("{}\0", command.str_flags()).as_ptr(),
        0,
        0,
        0,
    ) == api::Status::Err {
        return api::Status::Err;
    }
    return api::Status::Ok
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn Throttle_RedisCommand(
    ctx: *mut api::RedisModuleCtx,
    argv: *mut *mut api::RedisModuleString,
    argc: libc::c_int,
) -> api::Status {
    Command::harness(&ThrottleCommand {}, ctx, argv, argc)
}

// ThrottleCommand provides GCRA rate limiting as a command in Redis.
pub struct ThrottleCommand {}

impl Command for ThrottleCommand {
    // Should return the name of the command to be registered.
    fn name(&self) -> &'static str {
        "cl.throttle"
    }

    //noinspection RsTypeCheck
    // Run the command.
    fn run(&self, r: Redis, args: &[&str]) -> Result<(), SlicedError> {
        if args.len() != 5 && args.len() != 6 {
            return Err(error!(
                "Usage: {} <key> <max_burst> <count per period> \
                 <period> [<quantity>]",
                self.name()
            ));
        }

        // the first argument is command name "cl.throttle" (ignore it)
        let key = args[1];
        let max_burst = cmd::parse_i64(args[2])?;
        let count = cmd::parse_i64(args[3])?;
        let period = cmd::parse_i64(args[4])?;
        let quantity = match args.get(5) {
            Some(n) => cmd::parse_i64(n)?,
            None => 1,
        };

        // We reinitialize a new store and rate limiter every time this command
        // is run, but these structures don't have a huge overhead to them so
        // it's not that big of a problem.
        let mut store = store::InternalRedisStore::new(&r);
        let rate = cell::Rate::per_period(count, time::Duration::seconds(period));
        let mut limiter = cell::RateLimiter::new(
            &mut store,
            &cell::RateQuota {
                max_burst,
                max_rate: rate,
            },
        );

        let (throttled, rate_limit_result) = limiter.rate_limit(key, quantity)?;

        // Reply with an array containing rate limiting results. Note that
        // Redis' support for interesting data types is quite weak, so we have
        // to jam a few square pegs into round holes. It's a little messy, but
        // the interface comes out as pretty workable.
        r.reply_array(5)?;
        r.reply_integer(if throttled { 1 } else { 0 })?;
        r.reply_integer(rate_limit_result.limit)?;
        r.reply_integer(rate_limit_result.remaining)?;
        r.reply_integer(rate_limit_result.retry_after.num_seconds())?;
        r.reply_integer(rate_limit_result.reset_after.num_seconds())?;

        Ok(())
    }

    // Should return any flags to be registered with the name as a string
    // separated list. See the Redis module API documentation for a complete
    // list of the ones that are available.
    fn str_flags(&self) -> &'static str {
        "write"
    }
}