// Allow dead code in here in case I want to publish it as a crate at some
// point.
#![allow(dead_code)]

extern crate libc;

pub static POSTPONED_ARRAY_LEN: libc::c_int = -1;
pub static NO_EXPIRE: libc::c_int = -1;
//pub static POSITIVE_INFINITE: libc::c_int = std::m/

/// Error status return values
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Ok = 0,
    Err = 1,
}

// Rust can't link against C macros (#define) so we just redefine them here.
// There's a ~0 chance that any of these will ever change so it's pretty safe.
pub const REDISMODULE_APIVER_1: libc::c_int = 1;

bitflags! {
    pub struct KeyMode: libc::c_int {
        const READ = 1;
        const WRITE = (1 << 1);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyType {
    Empty = 0,
    String = 1,
    List = 2,
    Hash = 3,
    Set = 4,
    Zset = 5,
    Module = 6,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ListWhere {
    Head = 0,
    Tail = 1,
}

#[derive(Debug, PartialEq)]
pub enum ReplyType {
    Unknown = -1,
    String = 0,
    Error = 1,
    Integer = 2,
    Array = 3,
    Nil = 4,
}


bitflags! {
    pub struct ZaddFlag: libc::c_int {
        const XX = 1;
        const NX = (1 << 1);
        const ADDED = (1 << 2);
        const UPDATED = (1 << 3);
        const NOP = (1 << 4);
    }
}

bitflags! {
    pub struct HashFlag: libc::c_int {
        const NONE = 0;
        const NX = 1;
        const XX = (1 << 1);
        const CFIELDS = (1 << 2);
        const EXISTS = (1 << 3);
    }
}

bitflags! {
    pub struct ContextFlags: libc::c_int {
        /// The command is running in the context of a Lua script
        const LUA = 1;
        /// The command is running inside a Redis transaction
        const MULTI = (1 << 1);
        /// The instance is a master
        const MASTER = (1 << 2);
        /// The instance is a slave
        const SLAVE = (1 << 3);
        /// The instance is read-only (usually meaning it's a slave as well)
        const READONLY = (1 << 4);
        /// The instance is running in cluster mode
        const CLUSTER = (1 << 5);
        /// The instance has AOF enabled
        const AOF = (1 << 6);
        /// The instance has RDB enabled
        const RDB = (1 << 7);
        /// The instance has Maxmemory set
        const MAXMEMORY = (1 << 8);
        /// Maxmemory is set and has an eviction policy that may delete keys
        const EVICT = (1 << 9);
        /// Redis is out of memory according to the maxmemory flag.
        const OOM = (1 << 10);
        /// Less than 25% of memory available according to maxmemory.
        const OOM_WARNING = (1 << 11);
    }
}

bitflags! {
    pub struct NotifyFlags: libc::c_int {
        const GENERIC = (1 << 2);   // g
        const STRING = (1 << 3);    // $
        const LIST = (1 << 4);      // l
        const SET = (1 << 5);       // s
        const HASH = (1 << 6);      // h
        const ZSET = (1 << 7);      // z
        const EXPIRED = (1 << 8);   // x
        const EVICTED = (1 << 9);   // e
        const STREAM = (1 << 10);   // t
        const ALL = (Self::GENERIC.bits | Self::STRING.bits | Self::LIST.bits | Self::SET.bits | Self::HASH.bits | Self::ZSET.bits | Self::EXPIRED.bits | Self::EVICTED.bits | Self::STREAM.bits);
    }
}

pub static NODE_ID_LEN: libc::c_int = 40;

bitflags! {
    pub struct ClusterFlags: libc::c_int {
        const MYSELF = 1;
        const MASTER = (1 << 1);
        const SLAVE = (1 << 2);
        const PFAIL = (1 << 3);
        const FAIL = (1 << 4);
        const NOFAILOVER = (1 << 5);
    }
}

pub static ERRORMSG_WRONGTYPE: &str = "WRONGTYPE Operation against a key holding the wrong kind of value";

/// This type represents a timer handle, and is returned when a timer is
/// registered and used in order to invalidate a timer. It's just a 64 bit
/// number, because this is how each timer is represented inside the radix tree
/// of timers that are going to expire, sorted by expire time.
pub type RedisModuleTimerID = libc::uint64_t;


///
///
///
#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModule;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleCtx;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleKey;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleString;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleCallReply;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleIO;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleType;
//{
//    pub id: libc::uint64_t,
//    pub module: Option<RedisModule>,
//    pub rdb_load: RedisModuleTypeLoadFunc,
//    pub rdb_save: RedisModuleTypeSaveFunc,
//    pub aof_rewrite: RedisModuleTypeRewriteFunc,
//    pub mem_usage: RedisModuleTypeMemUsageFunc,
//    pub digest: RedisModuleTypeDigestFunc,
//    pub free: RedisModuleTypeFreeFunc,
//    pub name: [libc::c_char; 10],
//}


//#[allow(non_snake_case)]
//#[allow(unused_variables)]
////#[no_mangle]
//extern "C" fn Empty_RDBLoad(rdb: *mut RedisModuleIO,
//                            encver: libc::c_int) {}
//
//#[allow(non_snake_case)]
//#[allow(unused_variables)]
////#[no_mangle]
//extern "C" fn Empty_RDBSave(rdb: *mut RedisModuleIO,
//                            value: *mut u8) {}
//
//#[allow(non_snake_case)]
//#[allow(unused_variables)]
////#[no_mangle]
//extern "C" fn Empty_AOFRewrite(rdb: *mut RedisModuleIO,
//                               key: *mut RedisModuleString,
//                               value: *mut u8) {}
//
//#[allow(non_snake_case)]
//#[allow(unused_variables)]
////#[no_mangle]
//extern "C" fn Empty_MemUsage(rdb: *mut RedisModuleIO,
//                             key: *mut RedisModuleString,
//                             value: *mut u8) -> libc::size_t {
//    return 0;
//}
//
//#[allow(non_snake_case)]
//#[allow(unused_variables)]
////#[no_mangle]
//extern "C" fn Empty_Digest(digest: *mut RedisModuleDigest,
//                           value: *mut u8) {}

#[allow(non_snake_case)]
#[allow(unused_variables)]
//#[no_mangle]
extern "C" fn Empty_Free(value: *mut u8) {}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleDigest;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleBlockedClient;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleClusterInfo;

///
///
///
#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleTypeMethods {
    pub version: libc::uint64_t,
    pub rdb_load: Option<RedisModuleTypeLoadFunc>,
    pub rdb_save: Option<RedisModuleTypeSaveFunc>,
    pub aof_rewrite: Option<RedisModuleTypeRewriteFunc>,
    pub mem_usage: Option<RedisModuleTypeMemUsageFunc>,
    pub digest: Option<RedisModuleTypeDigestFunc>,
    pub free: Option<RedisModuleTypeFreeFunc>,
}

///
///
///
pub type RedisModuleCmdFunc = extern "C" fn(
    ctx: *mut RedisModuleCtx,
    argv: *mut *mut RedisModuleString,
    argc: libc::c_int,
) -> Status;

///
///
///
pub type RedisModuleDisconnectFunc = extern "C" fn(
    ctx: *mut RedisModuleCtx,
    bc: *mut RedisModuleBlockedClient,
);

///
///
///
pub type RedisModuleNotificationFunc = extern "C" fn(
    ctx: *mut RedisModuleCtx,
    rtype: libc::c_int,
    event: *mut u8,
    key: *mut RedisModuleString,
) -> libc::c_int;

///
///
///
pub type RedisModuleTypeLoadFunc = extern "C" fn(
    rdb: *mut RedisModuleIO,
    encver: libc::c_int,
);

///
///
///
pub type RedisModuleTypeSaveFunc = extern "C" fn(
    rdb: *mut RedisModuleIO,
    value: *mut u8,
);

///
///
///
pub type RedisModuleTypeRewriteFunc = extern "C" fn(
    rdb: *mut RedisModuleIO,
    key: *mut RedisModuleString,
    value: *mut u8,
);

///
///
///
pub type RedisModuleTypeMemUsageFunc = extern "C" fn(
    rdb: *mut RedisModuleIO,
    key: *mut RedisModuleString,
    value: *mut u8,
) -> libc::size_t;

///
///
///
pub type RedisModuleTypeDigestFunc = extern "C" fn(
    digest: *mut RedisModuleDigest,
    value: *mut u8,
);

///
///
///
pub type RedisModuleTypeFreeFunc = extern "C" fn(
    value: *mut u8,
);

///
///
///
pub type RedisFreePrivDataFunc = extern "C" fn(
    ctx: *mut RedisModuleCtx,
    value: *mut libc::c_void,
) -> *mut libc::c_void;

///
///
///
pub type RedisModuleClusterMessageReceiver = extern "C" fn(
    ctx: *mut RedisModuleCtx,
    sender_id: *mut u8,
    typ: libc::uint8_t,
    payload: *mut u8,
    len: libc::uint32_t,
);

///
///
///
pub type RedisModuleTimerProc = extern "C" fn(
    value: *mut libc::c_void,
);

///
/// RedisModule_Init
///
pub fn init(
    ctx: *mut RedisModuleCtx,
    modulename: *const u8,
    module_version: libc::c_int,
    api_version: libc::c_int,
) -> Status {
    unsafe { Export_RedisModule_Init(ctx, modulename, module_version, api_version) }
}

/// Return non-zero if a module command, that was declared with the
/// flag "getkeys-api", is called in a special way to get the keys positions
/// and not to get executed. Otherwise zero is returned.
pub fn is_keys_position_request(ctx: *mut RedisModuleCtx) -> bool {
    unsafe { RedisModule_IsKeysPositionRequest(ctx) != 0 }
}

/// When a module command is called in order to obtain the position of
/// keys, since it was flagged as "getkeys-api" during the registration,
/// the command implementation checks for this special call using the
/// RedisModule_IsKeysPositionRequest() API and uses this function in
/// order to report keys, like in the following example:
///
///     if (RedisModule_IsKeysPositionRequest(ctx)) {
///         RedisModule_KeyAtPos(ctx,1);
///         RedisModule_KeyAtPos(ctx,2);
///     }
///
///  Note: in the example below the get keys API would not be needed since
///  keys are at fixed positions. This interface is only used for commands
///  with a more complex structure.
pub fn key_at_pos(ctx: *mut RedisModuleCtx, pos: libc::c_int) {
    unsafe { RedisModule_KeyAtPos(ctx, pos) }
}

///
/// RedisModule_CallReplyType
///
pub fn call_reply_type(reply: *mut RedisModuleCallReply) -> ReplyType {
    unsafe { RedisModule_CallReplyType(reply) }
}

///
/// RedisModule_FreeCallReply
///
pub fn free_call_reply(reply: *mut RedisModuleCallReply) {
    unsafe { RedisModule_FreeCallReply(reply); }
}

///
/// RedisModule_CallReplyInteger
///
pub fn call_reply_integer(reply: *mut RedisModuleCallReply) -> libc::c_longlong {
    unsafe { RedisModule_CallReplyInteger(reply) }
}

///
/// RedisModule_CallReplyStringPtr
///
pub fn call_reply_string_ptr(
    str: *mut RedisModuleCallReply,
    len: *mut libc::size_t,
) -> *const u8 {
    unsafe { RedisModule_CallReplyStringPtr(str, len) }
}

/// Register a new command in the Redis server, that will be handled by
/// calling the function pointer 'func' using the RedisModule calling
/// convention. The function returns REDISMODULE_ERR if the specified command
/// name is already busy or a set of invalid flags were passed, otherwise
/// REDISMODULE_OK is returned and the new command is registered.
///
/// This function must be called during the initialization of the module
/// inside the RedisModule_OnLoad() function. Calling this function outside
/// of the initialization function is not defined.
///
/// The command function type is the following:
///
///      int MyCommand_RedisCommand(RedisModuleCtx *ctx, RedisModuleString **argv, int argc);
///
/// And is supposed to always return REDISMODULE_OK.
///
/// The set of flags 'strflags' specify the behavior of the command, and should
/// be passed as a C string compoesd of space separated words, like for
/// example "write deny-oom". The set of flags are:
///
/// * **"write"**:     The command may modify the data set (it may also read
///                    from it).
/// * **"readonly"**:  The command returns data from keys but never writes.
/// * **"admin"**:     The command is an administrative command (may change
///                    replication or perform similar tasks).
/// * **"deny-oom"**:  The command may use additional memory and should be
///                    denied during out of memory conditions.
/// * **"deny-script"**:   Don't allow this command in Lua scripts.
/// * **"allow-loading"**: Allow this command while the server is loading data.
///                        Only commands not interacting with the data set
///                        should be allowed to run in this mode. If not sure
///                        don't use this flag.
/// * **"pubsub"**:    The command publishes things on Pub/Sub channels.
/// * **"random"**:    The command may have different outputs even starting
///                    from the same input arguments and key values.
/// * **"allow-stale"**: The command is allowed to run on slaves that don't
///                      serve stale data. Don't use if you don't know what
///                      this means.
/// * **"no-monitor"**: Don't propoagate the command on monitor. Use this if
///                     the command has sensible data among the arguments.
/// * **"fast"**:      The command time complexity is not greater
///                    than O(log(N)) where N is the size of the collection or
///                    anything else representing the normal scalability
///                    issue with the command.
/// * **"getkeys-api"**: The command implements the interface to return
///                      the arguments that are keys. Used when start/stop/step
///                      is not enough because of the command syntax.
/// * **"no-cluster"**: The command should not register in Redis Cluster
///                     since is not designed to work with it because, for
///                     example, is unable to report the position of the
///                     keys, programmatically creates key names, or any
///                     other reason.
pub fn create_command(
    ctx: *mut RedisModuleCtx,
    name: *const u8,
    cmdfunc: Option<RedisModuleCmdFunc>,
    strflags: *const u8,
    firstkey: libc::c_int,
    lastkey: libc::c_int,
    keystep: libc::c_int,
) -> Status {
    unsafe {
        RedisModule_CreateCommand(
            ctx,
            name,
            cmdfunc,
            strflags,
            firstkey,
            lastkey,
            keystep,
        )
    }
}

/// Return non-zero if the module name is busy.
/// Otherwise zero is returned.
pub fn is_module_name_busy(name: *const u8) -> bool {
    unsafe { RedisModule_IsModuleNameBusy(name) != 0 }
}

/// Return the current UNIX time in milliseconds.
pub fn milliseconds() -> libc::c_longlong {
    unsafe { RedisModule_Milliseconds() }
}


/* --------------------------------------------------------------------------
 * String objects APIs
 * -------------------------------------------------------------------------- */


/// Create a new module string object. The returned string must be freed
/// with RedisModule_FreeString(), unless automatic memory is enabled.
///
/// The string is created by copying the `len` bytes starting
/// at `ptr`. No reference is retained to the passed buffer.
pub fn create_string(
    ctx: *mut RedisModuleCtx,
    ptr: *const u8,
    len: libc::size_t,
) -> *mut RedisModuleString {
    unsafe { RedisModule_CreateString(ctx, ptr, len) }
}

/// Like RedisModule_CreateString(), but creates a string starting from a long long
/// integer instead of taking a buffer and its length.
///
/// The returned string must be released with RedisModule_FreeString() or by
/// enabling automatic memory management.
pub fn create_string_from_long_long(
    ctx: *mut RedisModuleCtx,
    ll: libc::c_longlong) -> *mut RedisModuleString {
    unsafe { RedisModule_CreateStringFromLongLong(ctx, ll) }
}

/// Like RedisModule_CreatString(), but creates a string starting from another
/// RedisModuleString.
///
/// The returned string must be released with RedisModule_FreeString() or by
/// enabling automatic memory management.
pub fn create_string_from_string(
    ctx: *mut RedisModuleCtx,
    sstr: *mut RedisModuleString) -> *mut RedisModuleString {
    unsafe { RedisModule_CreateStringFromString(ctx, sstr) }
}

/// Every call to this function, will make the string 'str' requiring
/// an additional call to RedisModule_FreeString() in order to really
/// free the string. Note that the automatic freeing of the string obtained
/// enabling modules automatic memory management counts for one
/// RedisModule_FreeString() call (it is just executed automatically).
///
/// Normally you want to call this function when, at the same time
/// the following conditions are true:
///
/// 1) You have automatic memory management enabled.
/// 2) You want to create string objects.
/// 3) Those string objects you create need to live *after* the callback
///    function(for example a command implementation) creating them returns.
///
/// Usually you want this in order to store the created string object
/// into your own data structure, for example when implementing a new data
/// type.
///
/// Note that when memory management is turned off, you don't need
/// any call to RetainString() since creating a string will always result
/// into a string that lives after the callback function returns, if
/// no FreeString() call is performed.
pub fn retain_string(ctx: *mut RedisModuleCtx, sstr: *mut RedisModuleString) {
    unsafe { RedisModule_RetainString(ctx, sstr) }
}

/// Free a module string object obtained with one of the Redis modules API calls
/// that return new string objects.
///
/// It is possible to call this function even when automatic memory management
/// is enabled. In that case the string will be released ASAP and removed
/// from the pool of string to release at the end.
pub fn free_string(ctx: *mut RedisModuleCtx, str: *mut RedisModuleString) {
    unsafe { RedisModule_FreeString(ctx, str) }
}

/// Given a string module object, this function returns the string pointer
/// and length of the string. The returned pointer and length should only
/// be used for read only accesses and never modified.
pub fn string_ptr_len(str: *mut RedisModuleString,
                      len: *mut libc::size_t) -> *const u8 {
    unsafe { RedisModule_StringPtrLen(str, len) }
}


/* --------------------------------------------------------------------------
 * Higher level string operations
 * ------------------------------------------------------------------------- */

/// Convert the string into a long long integer, storing it at `*ll`.
/// Returns REDISMODULE_OK on success. If the string can't be parsed
/// as a valid, strict long long (no spaces before/after), REDISMODULE_ERR
/// is returned.
pub fn string_to_long_long(str: *mut RedisModuleString,
                           ll: *mut libc::c_longlong) -> Status {
    unsafe { RedisModule_StringToLongLong(str, ll) }
}

/// Convert the string into a double, storing it at `*d`.
/// Returns REDISMODULE_OK on success or REDISMODULE_ERR if the string is
/// not a valid string representation of a double value.
pub fn string_to_double(str: *mut RedisModuleString,
                        d: *mut libc::c_double) -> Status {
    unsafe { RedisModule_StringToDouble(str, d) }
}

/// Compare two string objects, returning -1, 0 or 1 respectively if
/// a < b, a == b, a > b. Strings are compared byte by byte as two
/// binary blobs without any encoding care / collation attempt.
pub fn string_compare(a: *mut RedisModuleString,
                      b: *mut RedisModuleString) -> libc::c_int {
    unsafe { RedisModule_StringCompare(a, b) }
}

/// Append the specified buffere to the string 'str'. The string must be a
/// string created by the user that is referenced only a single time, otherwise
/// REDISMODULE_ERR is returend and the operation is not performed.
pub fn string_append_buffer(ctx: *mut RedisModuleCtx,
                            str: *mut RedisModuleString,
                            buf: *const u8,
                            len: libc::size_t) -> Status {
    unsafe { RedisModule_StringAppendBuffer(ctx, str, buf, len) }
}


pub fn log(ctx: *mut RedisModuleCtx, level: *const u8, fmt: *const u8) {
    unsafe { RedisModule_Log(ctx, level, fmt) }
}


/* --------------------------------------------------------------------------
 * Reply APIs
 *
 * Most functions always return REDISMODULE_OK so you can use it with
 * 'return' in order to return from the command implementation with:
 *
 *     if (... some condition ...)
 *         return RM_ReplyWithLongLong(ctx,mycount);
 * -------------------------------------------------------------------------- */

/// Send an error about the number of arguments given to the command,
/// citing the command name in the error message.
///
/// Example:
///
///     if (argc != 3) return RedisModule_WrongArity(ctx);
pub fn wrong_arity(ctx: *mut RedisModuleCtx) -> Status {
    unsafe { RedisModule_WrongArity(ctx) }
}

/// Reply with an array type of 'len' elements. However 'len' other calls
/// to `ReplyWith*` style functions must follow in order to emit the elements
/// of the array.
///
/// When producing arrays with a number of element that is not known beforehand
/// the function can be called with the special count
/// REDISMODULE_POSTPONED_ARRAY_LEN, and the actual number of elements can be
/// later set with RedisModule_ReplySetArrayLength() (which will set the
/// latest "open" count if there are multiple ones).
///
/// The function always returns REDISMODULE_OK.
pub fn reply_with_array(ctx: *mut RedisModuleCtx,
                        len: libc::c_long) -> Status {
    unsafe { RedisModule_ReplyWithArray(ctx, len) }
}

/// Reply with the error 'err'.
///
/// Note that 'err' must contain all the error, including
/// the initial error code. The function only provides the initial "-", so
/// the usage is, for example:
///
///     RedisModule_ReplyWithError(ctx,"ERR Wrong Type");
///
/// and not just:
///
///     RedisModule_ReplyWithError(ctx,"Wrong Type");
///
/// The function always returns REDISMODULE_OK.
pub fn reply_with_error(ctx: *mut RedisModuleCtx,
                        err: *const u8) {
    unsafe { RedisModule_ReplyWithError(ctx, err) }
}

/// Send an integer reply to the client, with the specified long long value.
/// The function always returns REDISMODULE_OK.
pub fn reply_with_long_long(ctx: *mut RedisModuleCtx,
                            ll: libc::c_longlong) -> Status {
    unsafe { RedisModule_ReplyWithLongLong(ctx, ll) }
}

/// ion guarantees to always set the latest array length
/// that was created in a postponed way.
///
/// For example in order to output an array like [1,[10,20,30]] we
/// could write:
///
///      RedisModule_ReplyWithArray(ctx,REDISMODULE_POSTPONED_ARRAY_LEN);
///      RedisModule_ReplyWithLongLong(ctx,1);
///      RedisModule_ReplyWithArray(ctx,REDISMODULE_POSTPONED_ARRAY_LEN);
///      RedisModule_ReplyWithLongLong(ctx,10);
///      RedisModule_ReplyWithLongLong(ctx,20);
///      RedisModule_ReplyWithLongLong(ctx,30);
///      RedisModule_ReplySetArrayLength(ctx,3); // Set len of 10,20,30 array.
///      RedisModule_ReplySetArrayLength(ctx,2); // Set len of top array
///
/// Note that in the above example there is no reason to postpone the array
/// length, since we produce a fixed number of elements, but in the practice
/// the code may use an interator or other ways of creating the output so
/// that is not easy to calculate in advance the number of elements.
pub fn reply_set_array_length(ctx: *mut RedisModuleCtx, len: libc::c_long) {
    unsafe { RedisModule_ReplySetArrayLength(ctx, len) }
}

/// Reply with a bulk string, taking in input a C buffer pointer and length.
///
/// The function always returns REDISMODULE_OK.
pub fn reply_with_string_buffer(ctx: *mut RedisModuleCtx,
                                buf: *const u8,
                                len: libc::size_t) -> Status {
    unsafe { RedisModule_ReplyWithStringBuffer(ctx, buf, len) }
}


/// Reply with a bulk string, taking in input a RedisModuleString object.
///
/// The function always returns REDISMODULE_OK.
pub fn reply_with_string(
    ctx: *mut RedisModuleCtx,
    str: *mut RedisModuleString,
) -> Status {
    unsafe { RedisModule_ReplyWithString(ctx, str) }
}


/// Reply to the client with a NULL. In the RESP protocol a NULL is encoded
/// as the string "$-1\r\n".
///
/// The function always returns REDISMODULE_OK.
pub fn reply_with_null(
    ctx: *mut RedisModuleCtx
) -> Status {
    unsafe { RedisModule_ReplyWithNull(ctx) }
}


/// Reply exactly what a Redis command returned us with RedisModule_Call().
/// This function is useful when we use RedisModule_Call() in order to
/// execute some command, as we want to reply to the client exactly the
/// same reply we obtained by the command.
///
/// The function always returns REDISMODULE_OK.
pub fn reply_with_call_reply(
    ctx: *mut RedisModuleCtx,
    reply: *mut RedisModuleCallReply,
) -> Status {
    unsafe { RedisModule_ReplyWithCallReply(ctx, reply) }
}


/// Reply with a bulk string, taking in input a RedisModuleString object.
///
/// The function always returns REDISMODULE_OK.
pub fn reply_with_double(
    ctx: *mut RedisModuleCtx,
    d: libc::c_double,
) -> Status {
    unsafe { RedisModule_ReplyWithDouble(ctx, d) }
}


/* --------------------------------------------------------------------------
 * Commands replication API
 * -------------------------------------------------------------------------- */

/// Replicate the specified command and arguments to slaves and AOF, as effect
/// of execution of the calling command implementation.
///
/// The replicated commands are always wrapped into the MULTI/EXEC that
/// contains all the commands replicated in a given module command
/// execution. However the commands replicated with RedisModule_Call()
/// are the first items, the ones replicated with RedisModule_Replicate()
/// will all follow before the EXEC.
///
/// Modules should try to use one interface or the other.
///
/// This command follows exactly the same interface of RedisModule_Call(),
/// so a set of format specifiers must be passed, followed by arguments
/// matching the provided format specifiers.
///
/// Please refer to RedisModule_Call() for more information.
///
/// The command returns REDISMODULE_ERR if the format specifiers are invalid
/// or the command name does not belong to a known command.
pub fn replicate(
    ctx: *mut ::redis::api::RedisModuleCtx,
    cmdname: *const u8,
    fmt: *const u8,
) -> Status {
    unsafe { RedisModule_Replicate(ctx, cmdname, fmt) }
}

pub fn replicate_verbatim(ctx: *mut RedisModuleCtx) -> Status {
    unsafe { RedisModule_ReplicateVerbatim(ctx) }
}


/* --------------------------------------------------------------------------
 * DB and Key APIs -- Generic API
 * -------------------------------------------------------------------------- */

/// Return the ID of the current client calling the currently active module
/// command. The returned ID has a few guarantees:
///
/// 1. The ID is different for each different client, so if the same client
///    executes a module command multiple times, it can be recognized as
///    having the same ID, otherwise the ID will be different.
/// 2. The ID increases monotonically. Clients connecting to the server later
///    are guaranteed to get IDs greater than any past ID previously seen.
///
/// Valid IDs are from 1 to 2^64-1. If 0 is returned it means there is no way
/// to fetch the ID in the context the function was currently called.
///
pub fn get_client_id(ctx: *mut RedisModuleCtx) -> libc::c_longlong {
    unsafe { RedisModule_GetClientId(ctx) }
}

/// Return the currently selected DB.
pub fn get_selected_db(ctx: *mut RedisModuleCtx) -> libc::c_int {
    unsafe { RedisModule_GetSelectedDb(ctx) }
}

/// Return the current context's flags. The flags provide information on the
/// current request context (whether the client is a Lua script or in a MULTI),
/// and about the Redis instance in general, i.e replication and persistence.
///
/// The available flags are:
///
///  * REDISMODULE_CTX_FLAGS_LUA: The command is running in a Lua script
///
///  * REDISMODULE_CTX_FLAGS_MULTI: The command is running inside a transaction
///
///  * REDISMODULE_CTX_FLAGS_MASTER: The Redis instance is a master
///
///  * REDISMODULE_CTX_FLAGS_SLAVE: The Redis instance is a slave
///
///  * REDISMODULE_CTX_FLAGS_READONLY: The Redis instance is read-only
///
///  * REDISMODULE_CTX_FLAGS_CLUSTER: The Redis instance is in cluster mode
///
///  * REDISMODULE_CTX_FLAGS_AOF: The Redis instance has AOF enabled
///
///  * REDISMODULE_CTX_FLAGS_RDB: The instance has RDB enabled
///
///  * REDISMODULE_CTX_FLAGS_MAXMEMORY:  The instance has Maxmemory set
///
///  * REDISMODULE_CTX_FLAGS_EVICT:  Maxmemory is set and has an eviction
///    policy that may delete keys
///
///  * REDISMODULE_CTX_FLAGS_OOM: Redis is out of memory according to the
///    maxmemory setting.
///
///  * REDISMODULE_CTX_FLAGS_OOM_WARNING: Less than 25% of memory remains before
///                                       reaching the maxmemory level.
pub fn get_context_flags(ctx: *mut RedisModuleCtx) -> libc::c_int {
    unsafe { RedisModule_GetContextFlags(ctx) }
}

/// Change the currently selected DB. Returns an error if the id
/// is out of range.
///
/// Note that the client will retain the currently selected DB even after
/// the Redis command implemented by the module calling this function
/// returns.
///
/// If the module command wishes to change something in a different DB and
/// returns back to the original one, it should call RedisModule_GetSelectedDb()
/// before in order to restore the old DB number before returning.
pub fn select_db(ctx: *mut RedisModuleCtx, newid: libc::c_int) -> libc::c_int {
    unsafe { RedisModule_SelectDb(ctx, newid) }
}

/// Return an handle representing a Redis key, so that it is possible
/// to call other APIs with the key handle as argument to perform
/// operations on the key.
///
/// The return value is the handle repesenting the key, that must be
/// closed with RM_CloseKey().
///
/// If the key does not exist and WRITE mode is requested, the handle
/// is still returned, since it is possible to perform operations on
/// a yet not existing key (that will be created, for example, after
/// a list push operation). If the mode is just READ instead, and the
/// key does not exist, NULL is returned. However it is still safe to
/// call RedisModule_CloseKey() and RedisModule_KeyType() on a NULL
/// value.
pub fn open_key(
    ctx: *mut RedisModuleCtx,
    keyname: *mut RedisModuleString,
    mode: KeyMode,
) -> *mut RedisModuleKey {
    unsafe { RedisModule_OpenKey(ctx, keyname, mode) }
}

/// Close a key handle.
pub fn close_key(kp: *mut RedisModuleKey) {
    unsafe { RedisModule_CloseKey(kp) }
}

/// Return the type of the key. If the key pointer is NULL then
/// REDISMODULE_KEYTYPE_EMPTY is returned.
pub fn key_type(kp: *mut RedisModuleKey) -> KeyType {
    unsafe { RedisModule_KeyType(kp) }
}

/// Return the length of the value associated with the key.
/// For strings this is the length of the string. For all the other types
/// is the number of elements (just counting keys for hashes).
///
/// If the key pointer is NULL or the key is empty, zero is returned.
pub fn value_length(kp: *mut RedisModuleKey) -> libc::size_t {
    unsafe { RedisModule_ValueLength(kp) }
}

/// If the key is open for writing, remove it, and setup the key to
/// accept new writes as an empty key (that will be created on demand).
/// On success REDISMODULE_OK is returned. If the key is not open for
/// writing REDISMODULE_ERR is returned.
pub fn delete_key(kp: *mut RedisModuleKey) -> Status {
    unsafe { RedisModule_DeleteKey(kp) }
}

/// If the key is open for writing, unlink it (that is delete it in a 
/// non-blocking way, not reclaiming memory immediately) and setup the key to
/// accept new writes as an empty key (that will be created on demand).
/// On success REDISMODULE_OK is returned. If the key is not open for
/// writing REDISMODULE_ERR is returned.
pub fn unlink_key(kp: *mut RedisModuleKey) -> Status {
    unsafe { RedisModule_UnlinkKey(kp) }
}

/// Return the key expire value, as milliseconds of remaining TTL.
/// If no TTL is associated with the key or if the key is empty,
/// REDISMODULE_NO_EXPIRE is returned.
pub fn get_expire(key: *mut RedisModuleKey) -> libc::c_longlong {
    unsafe { RedisModule_GetExpire(key) }
}

/// Set a new expire for the key. If the special expire
/// REDISMODULE_NO_EXPIRE is set, the expire is cancelled if there was
/// one (the same as the PERSIST command).
///
/// Note that the expire must be provided as a positive integer representing
/// the number of milliseconds of TTL the key should have.
///
/// The function returns REDISMODULE_OK on success or REDISMODULE_ERR if
/// the key was not open for writing or is an empty key.
pub fn set_expire(key: *mut RedisModuleKey,
                  expire: libc::c_longlong) -> Status {
    unsafe { RedisModule_SetExpire(key, expire) }
}

/* --------------------------------------------------------------------------
 * Key API for String type
 * -------------------------------------------------------------------------- */

/// If the key is open for writing, set the specified string 'str' as the
/// value of the key, deleting the old value if any.
/// On success REDISMODULE_OK is returned. If the key is not open for
/// writing or there is an active iterator, REDISMODULE_ERR is returned.
pub fn string_set(key: *mut RedisModuleKey,
                  str: *mut RedisModuleString) -> Status {
    unsafe { RedisModule_StringSet(key, str) }
}

/// Prepare the key associated string value for DMA access, and returns
/// a pointer and size (by reference), that the user can use to read or
/// modify the string in-place accessing it directly via pointer.
///
/// The 'mode' is composed by bitwise OR-ing the following flags:
///
///     REDISMODULE_READ -- Read access
///     REDISMODULE_WRITE -- Write access
///
/// If the DMA is not requested for writing, the pointer returned should
/// only be accessed in a read-only fashion.
///
/// On error (wrong type) NULL is returned.
///
/// DMA access rules:
///
/// 1. No other key writing function should be called since the moment
/// the pointer is obtained, for all the time we want to use DMA access
/// to read or modify the string.
///
/// 2. Each time RM_StringTruncate() is called, to continue with the DMA
/// access, RM_StringDMA() should be called again to re-obtain
/// a new pointer and length.
///
/// 3. If the returned pointer is not NULL, but the length is zero, no
/// byte can be touched (the string is empty, or the key itself is empty)
/// so a RM_StringTruncate() call should be used if there is to enlarge
/// the string, and later call StringDMA() again to get the pointer.
pub fn string_dma(
    key: *mut RedisModuleKey,
    len: *mut libc::size_t,
    mode: KeyMode,
) -> *const u8 {
    unsafe { RedisModule_StringDMA(key, len, mode) }
}

/// If the string is open for writing and is of string type, resize it, padding
/// with zero bytes if the new length is greater than the old one.
///
/// After this call, RM_StringDMA() must be called again to continue
/// DMA access with the new pointer.
///
/// The function returns REDISMODULE_OK on success, and REDISMODULE_ERR on
/// error, that is, the key is not open for writing, is not a string
/// or resizing for more than 512 MB is requested.
///
/// If the key is empty, a string key is created with the new string value
/// unless the new length value requested is zero.
pub fn string_truncate(key: *mut RedisModuleKey, newlen: libc::size_t) -> Status {
    unsafe { RedisModule_StringTruncate(key, newlen) }
}


/* --------------------------------------------------------------------------
 * Key API for List type
 * -------------------------------------------------------------------------- */

/// Push an element into a list, on head or tail depending on 'where' argumnet.
/// If the key pointer is about an empty key opened for writing, the key
/// is created. On error (key opened for read-only operations or of the wrong
/// type) REDISMODULE_ERR is returned, otherwise REDISMODULE_OK is returned.
pub fn list_push(key: *mut RedisModuleKey,
                 wwhere: ListWhere,
                 ele: *mut RedisModuleString) -> Status {
    unsafe { RedisModule_ListPush(key, wwhere, ele) }
}

/// Pop an element from the list, and returns it as a module string object
/// that the user should be free with RM_FreeString() or by enabling
/// automatic memory. 'where' specifies if the element should be popped from
/// head or tail. The command returns NULL if:
/// 1) The list is empty.
/// 2) The key was not open for writing.
/// 3) The key is not a list.
pub fn list_pop(key: *mut RedisModuleKey,
                wwhere: ListWhere) -> *mut RedisModuleString {
    unsafe { RedisModule_ListPop(key, wwhere) }
}


/* --------------------------------------------------------------------------
 * Modules data types
 *
 * When String DMA or using existing data structures is not enough, it is
 * possible to create new data types from scratch and export them to
 * Redis. The module must provide a set of callbacks for handling the
 * new values exported (for example in order to provide RDB saving/loading,
 * AOF rewrite, and so forth). In this section we define this API.
 * -------------------------------------------------------------------------- */

/// Register a new data type exported by the module. The parameters are the
/// following. Please for in depth documentation check the modules API
/// documentation, especially the TYPES.md file.
///
/// * **name**: A 9 characters data type name that MUST be unique in the Redis
///   Modules ecosystem. Be creative... and there will be no collisions. Use
///   the charset A-Z a-z 9-0, plus the two "-_" characters. A good
///   idea is to use, for example `<typename>-<vendor>`. For example
///   "tree-AntZ" may mean "Tree data structure by @antirez". To use both
///   lower case and upper case letters helps in order to prevent collisions.
/// * **encver**: Encoding version, which is, the version of the serialization
///   that a module used in order to persist data. As long as the "name"
///   matches, the RDB loading will be dispatched to the type callbacks
///   whatever 'encver' is used, however the module can understand if
///   the encoding it must load are of an older version of the module.
///   For example the module "tree-AntZ" initially used encver=0. Later
///   after an upgrade, it started to serialize data in a different format
///   and to register the type with encver=1. However this module may
///   still load old data produced by an older version if the rdb_load
///   callback is able to check the encver value and act accordingly.
///   The encver must be a positive value between 0 and 1023.
/// * **typemethods_ptr** is a pointer to a RedisModuleTypeMethods structure
///   that should be populated with the methods callbacks and structure
///   version, like in the following example:
///
///      RedisModuleTypeMethods tm = {
///          .version = REDISMODULE_TYPE_METHOD_VERSION,
///          .rdb_load = myType_RDBLoadCallBack,
///          .rdb_save = myType_RDBSaveCallBack,
///          .aof_rewrite = myType_AOFRewriteCallBack,
///          .free = myType_FreeCallBack,
///
///          // Optional fields
///          .digest = myType_DigestCallBack,
///          .mem_usage = myType_MemUsageCallBack,
///      }
///
/// * **rdb_load**: A callback function pointer that loads data from RDB files.
/// * **rdb_save**: A callback function pointer that saves data to RDB files.
/// * **aof_rewrite**: A callback function pointer that rewrites data as commands.
/// * **digest**: A callback function pointer that is used for `DEBUG DIGEST`.
/// * **free**: A callback function pointer that can free a type value.
///
/// The **digest* and **mem_usage** methods should currently be omitted since
/// they are not yet implemented inside the Redis modules core.
///
/// Note: the module name "AAAAAAAAA" is reserved and produces an error, it
/// happens to be pretty lame as well.
///
/// If there is already a module registering a type with the same name,
/// and if the module name or encver is invalid, NULL is returned.
/// Otherwise the new type is registered into Redis, and a reference of
/// type RedisModuleType is returned: the caller of the function should store
/// this reference into a gobal variable to make future use of it in the
/// modules type API, since a single module may register multiple types.
/// Example code fragment:
///
///      static RedisModuleType *BalancedTreeType;
///
///      int RedisModule_OnLoad(RedisModuleCtx *ctx) {
///          // some code here ...
///          BalancedTreeType = RM_CreateDataType(...);
///      }
///
pub fn create_data_type(ctx: *mut RedisModuleCtx,
                        name: *const u8,
                        encver: libc::c_int,
                        rdb_load: Option<RedisModuleTypeLoadFunc>,
                        rdb_save: Option<RedisModuleTypeSaveFunc>,
                        aof_rewrite: Option<RedisModuleTypeRewriteFunc>,
                        mem_usage: Option<RedisModuleTypeMemUsageFunc>,
                        digest: Option<RedisModuleTypeDigestFunc>,
                        freefn: Option<RedisModuleTypeFreeFunc>) -> *mut RedisModuleType {
    unsafe {
        Export_RedisModule_CreateDataType(ctx,
                                          name,
                                          encver,
                                          rdb_load,
                                          rdb_save,
                                          aof_rewrite,
                                          mem_usage,
                                          digest,
                                          freefn)
    }
}

/* --------------------------------------------------------------------------
 * RDB loading and saving functions
 * -------------------------------------------------------------------------- */

// TODO: Implement RDB API

/* --------------------------------------------------------------------------
 * Key digest API (DEBUG DIGEST interface for modules types)
 * -------------------------------------------------------------------------- */

// TODO: Implement Key digest API

/* --------------------------------------------------------------------------
 * AOF API for modules data types
 * -------------------------------------------------------------------------- */

// TODO: Implement AOF API

/* --------------------------------------------------------------------------
 * Logging
 * -------------------------------------------------------------------------- */

//pub fn log_raw(module: *mut RedisModule, )
// TODO: Implement Logging API


/* --------------------------------------------------------------------------
 * Blocking clients from modules
 * -------------------------------------------------------------------------- */

/// Block a client in the context of a blocking command, returning an handle
/// which will be used, later, in order to unblock the client with a call to
/// RedisModule_UnblockClient(). The arguments specify callback functions
/// and a timeout after which the client is unblocked.
///
/// The callbacks are called in the following contexts:
///
///     reply_callback:  called after a successful RedisModule_UnblockClient()
///                      call in order to reply to the client and unblock it.
///
///     reply_timeout:   called when the timeout is reached in order to send an
///                      error to the client.
///
///     free_privdata:   called in order to free the privata data that is passed
///                      by RedisModule_UnblockClient() call.
pub fn block_client(ctx: *mut RedisModuleCtx,
                    reply_callback: Option<RedisModuleCmdFunc>,
                    timeout_callback: Option<RedisModuleCmdFunc>,
                    free_privdata: Option<RedisFreePrivDataFunc>,
                    timeout_ms: libc::c_longlong,
) -> *mut RedisModuleBlockedClient {
    unsafe { RedisModule_BlockClient(ctx, reply_callback, timeout_callback, free_privdata, timeout_ms) }
}

/// Abort a blocked client blocking operation: the client will be unblocked
/// without firing any callback
pub fn abort_block(bc: *mut RedisModuleBlockedClient) -> Status {
    unsafe { RedisModule_AbortBlock(bc) }
}

/// Unblock a client blocked by `RedisModule_BlockedClient`. This will trigger
/// the reply callbacks to be called in order to reply to the client.
/// The 'privdata' argument will be accessible by the reply callback, so
/// the caller of this function can pass any value that is needed in order to
/// actually reply to the client.
///
/// A common usage for 'privdata' is a thread that computes something that
/// needs to be passed to the client, included but not limited some slow
/// to compute reply or some reply obtained via networking.
///
/// Note: this function can be called from threads spawned by the module. */
pub fn unblock_client(bc: *mut RedisModuleBlockedClient, privdata: *mut u8) -> Status {
    unsafe { RedisModule_UnblockClient(bc, privdata as *mut libc::c_void) }
}

/// Set a callback that will be called if a blocked client disconnects
/// before the module has a chance to call RedisModule_UnblockClient()
///
/// Usually what you want to do there, is to cleanup your module state
/// so that you can call RedisModule_UnblockClient() safely, otherwise
/// the client will remain blocked forever if the timeout is large.
///
/// Notes:
///
/// 1. It is not safe to call Reply* family functions here, it is also
///    useless since the client is gone.
///
/// 2. This callback is not called if the client disconnects because of
///    a timeout. In such a case, the client is unblocked automatically
///    and the timeout callback is called.
pub fn set_disconnect_callback(bc: *mut RedisModuleBlockedClient, callback: Option<RedisModuleDisconnectFunc>) {
    unsafe { RedisModule_SetDisconnectCallback(bc, callback) }
}

/// Return non-zero if a module command was called in order to fill the
/// reply for a blocked client.
pub fn is_blocked_reply_request(ctx: *mut RedisModuleCtx) -> bool {
    unsafe { RedisModule_IsBlockedReplyRequest(ctx) != 0 }
}

/// Return non-zero if a module command was called in order to fill the
/// reply for a blocked client that timed out.
pub fn is_blocked_timeout_request(ctx: *mut RedisModuleCtx) -> bool {
    unsafe { RedisModule_IsBlockedTimeoutRequest(ctx) != 0 }
}

/// Get the privata data set by RedisModule_UnblockClient()
pub fn get_blocked_client_private_data(ctx: *mut RedisModuleCtx) -> *mut libc::c_void {
    unsafe { RedisModule_GetBlockedClientPrivateData(ctx) }
}

/// Get the blocked client associated with a given context.
/// This is useful in the reply and timeout callbacks of blocked clients,
/// before sometimes the module has the blocked client handle references
/// around, and wants to cleanup it.
pub fn get_blocked_client_handle(ctx: *mut RedisModuleCtx) -> *mut RedisModuleBlockedClient {
    unsafe { RedisModule_GetBlockedClientHandle(ctx) }
}

/// Return true if when the free callback of a blocked client is called,
/// the reason for the client to be unblocked is that it disconnected
/// while it was blocked.
pub fn is_blocked_client_disconnected(ctx: *mut RedisModuleCtx) -> bool {
    unsafe { RedisModule_BlockedClientDisconnected(ctx) != 0 }
}

/* --------------------------------------------------------------------------
 * Thread Safe Contexts
 * -------------------------------------------------------------------------- */

/// Return a context which can be used inside threads to make Redis context
/// calls with certain modules APIs. If 'bc' is not NULL then the module will
/// be bound to a blocked client, and it will be possible to use the
/// `RedisModule_Reply*` family of functions to accumulate a reply for when the
/// client will be unblocked. Otherwise the thread safe context will be
/// detached by a specific client.
///
/// To call non-reply APIs, the thread safe context must be prepared with:
///
///     RedisModule_ThreadSafeCallStart(ctx);
///     ... make your call here ...
///     RedisModule_ThreadSafeCallStop(ctx);
///
/// This is not needed when using `RedisModule_Reply*` functions, assuming
/// that a blocked client was used when the context was created, otherwise
/// no RedisModule_Reply* call should be made at all.
///
/// TODO: thread safe contexts do not inherit the blocked client
/// selected database.
pub fn get_thread_safe_context(bc: *mut RedisModuleBlockedClient) -> *mut RedisModuleCtx {
    unsafe { RedisModule_GetThreadSafeContext(bc) }
}

/// Release a thread safe context.
pub fn free_thread_safe_context(ctx: *mut RedisModuleCtx) {
    unsafe { RedisModule_FreeThreadSafeContext(ctx) }
}

/// Acquire the server lock before executing a thread safe API call.
/// This is not needed for `RedisModule_Reply*` calls when there is
/// a blocked client connected to the thread safe context.
pub fn thread_safe_context_lock(ctx: *mut RedisModuleCtx) {
    unsafe { RedisModule_ThreadSafeContextLock(ctx) }
}

/// Release the server lock after a thread safe API call was executed.
pub fn thread_safe_context_unlock(ctx: *mut RedisModuleCtx) {
    unsafe { RedisModule_ThreadSafeContextUnlock(ctx) }
}


/* --------------------------------------------------------------------------
 * Module Keyspace Notifications API
 * -------------------------------------------------------------------------- */

/// Subscribe to keyspace notifications. This is a low-level version of the
/// keyspace-notifications API. A module cand register callbacks to be notified
/// when keyspce events occur.
///
/// Notification events are filtered by their type (string events, set events,
/// etc), and the subsriber callback receives only events that match a specific
/// mask of event types.
///
/// When subscribing to notifications with RedisModule_SubscribeToKeyspaceEvents 
/// the module must provide an event type-mask, denoting the events the subscriber
/// is interested in. This can be an ORed mask of any of the following flags:
///
///  - REDISMODULE_NOTIFY_GENERIC: Generic commands like DEL, EXPIRE, RENAME
///  - REDISMODULE_NOTIFY_STRING: String events
///  - REDISMODULE_NOTIFY_LIST: List events
///  - REDISMODULE_NOTIFY_SET: Set events
///  - REDISMODULE_NOTIFY_HASH: Hash events
///  - REDISMODULE_NOTIFY_ZSET: Sorted Set events
///  - REDISMODULE_NOTIFY_EXPIRED: Expiration events
///  - REDISMODULE_NOTIFY_EVICTED: Eviction events
///  - REDISMODULE_NOTIFY_STREAM: Stream events
///  - REDISMODULE_NOTIFY_ALL: All events
///
/// We do not distinguish between key events and keyspace events, and it is up
/// to the module to filter the actions taken based on the key.
///
/// The subscriber signature is:
///
///   int (*RedisModuleNotificationFunc) (RedisModuleCtx/ctx, int type,
///                                       const char/event,
///                                       RedisModuleString/key);
///
/// `type` is the event type bit, that must match the mask given at registration
/// time. The event string is the actual command being executed, and key is the
/// relevant Redis key.
///
/// Notification callback gets executed with a redis context that can not be
/// used to send anything to the client, and has the db number where the event
/// occured as its selected db number.
///
/// Notice that it is not necessary to enable norifications in redis.conf for
/// module notifications to work.
///
/// Warning: the notification callbacks are performed in a synchronous manner,
/// so notification callbacks must to be fast, or they would slow Redis down.
/// If you need to take long actions, use threads to offload them.
///
/// See https://redis.io/topics/notifications for more information.
pub fn subscribe_to_keyspace_events(ctx: *mut RedisModuleCtx,
                                    types: NotifyFlags,
                                    callback: Option<RedisModuleNotificationFunc>) -> Status {
    unsafe { RedisModule_SubscribeToKeyspaceEvents(ctx, types, callback) }
}


/* --------------------------------------------------------------------------
 * Modules Timers API
 *
 * Module timers are an high precision "green timers" abstraction where
 * every module can register even millions of timers without problems, even if
 * the actual event loop will just have a single timer that is used to awake the
 * module timers subsystem in order to process the next event.
 *
 * All the timers are stored into a radix tree, ordered by expire time, when
 * the main Redis event loop timer callback is called, we try to process all
 * the timers already expired one after the other. Then we re-enter the event
 * loop registering a timer that will expire when the next to process module
 * timer will expire.
 *
 * Every time the list of active timers drops to zero, we unregister the
 * main event loop timer, so that there is no overhead when such feature is
 * not used.
 * -------------------------------------------------------------------------- */

/// Create a new timer that will fire after `period` milliseconds, and will call
/// the specified function using `data` as argument. The returned timer ID can be
/// used to get information from the timer or to stop it before it fires.
pub fn create_timer(ctx: *mut RedisModuleCtx,
                    period: libc::c_longlong,
                    callback: Option<RedisModuleTimerProc>,
                    data: *mut libc::c_void) -> RedisModuleTimerID {
    unsafe { RedisModule_CreateTimer(ctx, period, callback, data) }
}

/// Stop a timer, returns REDISMODULE_OK if the timer was found, belonged to the
/// calling module, and was stoped, otherwise REDISMODULE_ERR is returned.
/// If not NULL, the data pointer is set to the value of the data argument when
/// the timer was created.
pub fn stop_timer(ctx: *mut RedisModuleCtx,
                  id: RedisModuleTimerID,
                  data: *mut *mut libc::c_void) -> Status {
    unsafe { RedisModule_StopTimer(ctx, id, data) }
}

/// Obtain information about a timer: its remaining time before firing
/// (in milliseconds), and the private data pointer associated with the timer.
/// If the timer specified does not exist or belongs to a different module
/// no information is returned and the function returns REDISMODULE_ERR, otherwise
/// REDISMODULE_OK is returned. The argumnets remaining or data can be NULL if
/// the caller does not need certain information.
pub fn get_timer_info(ctx: *mut RedisModuleCtx,
                      id: RedisModuleTimerID,
                      remaining: libc::uint64_t,
                      data: *mut *mut libc::c_void) -> Status {
    unsafe { RedisModule_GetTimerInfo(ctx, id, remaining, data) }
}


// Redis doesn't make this easy for us by exporting a library, so instead what
// we do is bake redismodule.h's symbols into a library of our construction
// during build and link against that. See build.rs for details.
#[allow(improper_ctypes)]
#[allow(non_snake_case)]
#[link(name = "redismodule", kind = "static")]
extern "C" {
    ///
    /// Taps into the C helper shim
    ///
    pub fn Export_RedisModule_Init(
        ctx: *mut RedisModuleCtx,
        modulename: *const u8,
        module_version: libc::c_int,
        api_version: libc::c_int,
    ) -> Status;

    ///
    /// Taps into the C helper shim
    ///
    pub fn Export_RedisModule_CreateDataType(
        ctx: *mut RedisModuleCtx,
        name: *const u8,
        encver: libc::c_int,
        rdb_load: Option<RedisModuleTypeLoadFunc>,
        rdb_save: Option<RedisModuleTypeSaveFunc>,
        aof_rewrite: Option<RedisModuleTypeRewriteFunc>,
        mem_usage: Option<RedisModuleTypeMemUsageFunc>,
        digest: Option<RedisModuleTypeDigestFunc>,
        free: Option<RedisModuleTypeFreeFunc>,
    ) -> *mut RedisModuleType;

    static RedisModule_IsKeysPositionRequest:
    extern "C" fn(ctx: *mut RedisModuleCtx) -> libc::c_int;

    static RedisModule_KeyAtPos:
    extern "C" fn(ctx: *mut RedisModuleCtx, pos: libc::c_int);

    static RedisModule_IsModuleNameBusy:
    extern "C" fn(ctx: *const u8) -> libc::c_int;

    pub static RedisModule_CreateDataType:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  name: *const u8,
                  encver: libc::c_int,
                  typemethods: *const RedisModuleTypeMethods) -> Option<RedisModuleType>;

    pub static RedisModule_Milliseconds:
    extern "C" fn() -> libc::c_longlong;

    pub static Export_RedisModule_Alloc:
    extern "C" fn(size: libc::size_t) -> *mut libc::c_void;

    pub static RedisModule_Alloc:
    extern "C" fn(size: libc::size_t) -> *mut libc::c_void;

    pub static RedisModule_Realloc:
    extern "C" fn(ptr: *mut u8, size: libc::size_t) -> *mut u8;


    pub static RedisModule_Free:
    extern "C" fn(ptr: *mut u8);


    static RedisModule_CallReplyType:
    extern "C" fn(reply: *mut RedisModuleCallReply) -> ReplyType;

    static RedisModule_FreeCallReply: extern "C" fn(reply: *mut RedisModuleCallReply);

    static RedisModule_CallReplyInteger:
    extern "C" fn(reply: *mut RedisModuleCallReply) -> libc::c_longlong;

    static RedisModule_CallReplyStringPtr:
    extern "C" fn(str: *mut RedisModuleCallReply, len: *mut libc::size_t) -> *const u8;

    static RedisModule_CloseKey: extern "C" fn(kp: *mut RedisModuleKey);

    static RedisModule_KeyType: extern "C" fn(kp: *mut RedisModuleKey) -> KeyType;

    static RedisModule_ValueLength: extern "C" fn(kp: *mut RedisModuleKey) -> libc::size_t;

    static RedisModule_DeleteKey: extern "C" fn(kp: *mut RedisModuleKey) -> Status;

    static RedisModule_UnlinkKey: extern "C" fn(kp: *mut RedisModuleKey) -> Status;

    static RedisModule_GetExpire: extern "C" fn(kp: *mut RedisModuleKey) -> libc::c_longlong;

    pub static RedisModule_CreateCommand:
    extern "C" fn(
        ctx: *mut RedisModuleCtx,
        name: *const u8,
        cmdfunc: Option<RedisModuleCmdFunc>,
        strflags: *const u8,
        firstkey: libc::c_int,
        lastkey: libc::c_int,
        keystep: libc::c_int,
    ) -> Status;

    static RedisModule_CreateString:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  ptr: *const u8,
                  len: libc::size_t) -> *mut RedisModuleString;

    static RedisModule_CreateStringFromLongLong:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  ll: libc::c_longlong) -> *mut RedisModuleString;

    static RedisModule_CreateStringFromString:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  str: *mut RedisModuleString) -> *mut RedisModuleString;

    static RedisModule_RetainString:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  str: *mut RedisModuleString);

    static RedisModule_FreeString:
    extern "C" fn(ctx: *mut RedisModuleCtx, str: *mut RedisModuleString);

    static RedisModule_StringToLongLong:
    extern "C" fn(str: *mut RedisModuleString, ll: *mut libc::c_longlong) -> Status;

    static RedisModule_StringToDouble:
    extern "C" fn(str: *mut RedisModuleString, d: *mut libc::c_double) -> Status;

    static RedisModule_StringCompare:
    extern "C" fn(a: *mut RedisModuleString, b: *mut RedisModuleString) -> libc::c_int;


    static RedisModule_StringAppendBuffer:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  str: *mut RedisModuleString,
                  buf: *const u8,
                  len: libc::size_t) -> Status;

    static RedisModule_GetClientId:
    extern "C" fn(ctx: *mut RedisModuleCtx) -> libc::c_longlong;

    static RedisModule_GetSelectedDb:
    extern "C" fn(ctx: *mut RedisModuleCtx) -> libc::c_int;

    static RedisModule_GetContextFlags:
    extern "C" fn(ctx: *mut RedisModuleCtx) -> libc::c_int;

    static RedisModule_SelectDb:
    extern "C" fn(ctx: *mut RedisModuleCtx, newid: libc::c_int) -> libc::c_int;

    static RedisModule_Log:
    extern "C" fn(ctx: *mut RedisModuleCtx, level: *const u8, fmt: *const u8);

    static RedisModule_OpenKey:
    extern "C" fn(
        ctx: *mut RedisModuleCtx,
        keyname: *mut RedisModuleString,
        mode: KeyMode,
    ) -> *mut RedisModuleKey;

    static RedisModule_WrongArity:
    extern "C" fn(ctx: *mut RedisModuleCtx) -> Status;

    static RedisModule_ReplyWithArray:
    extern "C" fn(ctx: *mut RedisModuleCtx, len: libc::c_long) -> Status;

    static RedisModule_ReplyWithError:
    extern "C" fn(ctx: *mut RedisModuleCtx, err: *const u8);

    static RedisModule_ReplyWithLongLong:
    extern "C" fn(ctx: *mut RedisModuleCtx, ll: libc::c_longlong) -> Status;

    static RedisModule_ReplyWithString:
    extern "C" fn(ctx: *mut RedisModuleCtx, str: *mut RedisModuleString) -> Status;

    static RedisModule_ReplySetArrayLength:
    extern "C" fn(ctx: *mut RedisModuleCtx, str: libc::c_long);

    static RedisModule_ReplyWithStringBuffer:
    extern "C" fn(ctx: *mut RedisModuleCtx, buf: *const u8, len: libc::size_t) -> Status;

    static RedisModule_ReplyWithNull:
    extern "C" fn(ctx: *mut RedisModuleCtx) -> Status;

    static RedisModule_ReplyWithCallReply:
    extern "C" fn(ctx: *mut RedisModuleCtx, reply: *mut RedisModuleCallReply) -> Status;

    static RedisModule_ReplyWithDouble:
    extern "C" fn(ctx: *mut RedisModuleCtx, d: libc::c_double) -> Status;


    pub static RedisModule_Replicate:
    extern "C" fn(
        ctx: *mut ::redis::api::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
    ) -> Status;

    static RedisModule_ReplicateVerbatim:
    extern "C" fn(ctx: *mut RedisModuleCtx) -> Status;


    static RedisModule_SetExpire:
    extern "C" fn(key: *mut RedisModuleKey, expire: libc::c_longlong) -> Status;

    static RedisModule_StringDMA:
    extern "C" fn(key: *mut RedisModuleKey, len: *mut libc::size_t, mode: KeyMode) -> *const u8;

    static RedisModule_StringTruncate:
    extern "C" fn(key: *mut RedisModuleKey, newlen: libc::size_t) -> Status;


    static RedisModule_ListPush:
    extern "C" fn(key: *mut RedisModuleKey,
                  wwhere: ListWhere,
                  ele: *mut RedisModuleString) -> Status;

    static RedisModule_ListPop:
    extern "C" fn(key: *mut RedisModuleKey,
                  wwhere: ListWhere) -> *mut RedisModuleString;



    static RedisModule_StringPtrLen:
    extern "C" fn(str: *mut RedisModuleString, len: *mut libc::size_t) -> *const u8;

    static RedisModule_StringSet:
    extern "C" fn(key: *mut RedisModuleKey, str: *mut RedisModuleString) -> Status;

    static RedisModule_Call:
    extern "C" fn(
        ctx: *mut RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        args: *const *mut RedisModuleString,
    ) -> *mut RedisModuleCallReply;


    static RedisModule_BlockClient:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  reply_callback: Option<RedisModuleCmdFunc>,
                  timeout_callback: Option<RedisModuleCmdFunc>,
                  free_privdata: Option<RedisFreePrivDataFunc>,
                  timeout_ms: libc::c_longlong,
    ) -> *mut RedisModuleBlockedClient;

    static RedisModule_AbortBlock:
    extern "C" fn(bc: *mut RedisModuleBlockedClient) -> Status;

    static RedisModule_UnblockClient:
    extern "C" fn(bc: *mut RedisModuleBlockedClient,
                  privdata: *mut libc::c_void) -> Status;

    static RedisModule_SetDisconnectCallback:
    extern "C" fn(bc: *mut RedisModuleBlockedClient,
                  callback: Option<RedisModuleDisconnectFunc>);

    static RedisModule_IsBlockedReplyRequest:
    extern "C" fn(ctx: *mut RedisModuleCtx) -> libc::c_int;

    static RedisModule_IsBlockedTimeoutRequest:
    extern "C" fn(ctx: *mut RedisModuleCtx) -> libc::c_int;

    static RedisModule_GetBlockedClientPrivateData:
    extern "C" fn(ctx: *mut RedisModuleCtx) -> *mut libc::c_void;

    static RedisModule_GetBlockedClientHandle:
    extern "C" fn(ctx: *mut RedisModuleCtx) -> *mut RedisModuleBlockedClient;

    static RedisModule_BlockedClientDisconnected:
    extern "C" fn(ctx: *mut RedisModuleCtx) -> libc::c_int;

    static RedisModule_GetThreadSafeContext:
    extern "C" fn(bc: *mut RedisModuleBlockedClient) -> *mut RedisModuleCtx;

    static RedisModule_FreeThreadSafeContext:
    extern "C" fn(ctx: *mut RedisModuleCtx);

    static RedisModule_ThreadSafeContextLock:
    extern "C" fn(ctx: *mut RedisModuleCtx);

    static RedisModule_ThreadSafeContextUnlock:
    extern "C" fn(ctx: *mut RedisModuleCtx);

    fn Export_RedisModule_SubscribeToKeyspaceEvents(
        ctx: *mut RedisModuleCtx,
        types: libc::c_int,
        callback: Option<RedisModuleNotificationFunc>) -> libc::c_int;

    static RedisModule_SubscribeToKeyspaceEvents:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  types: NotifyFlags,
                  callback: Option<RedisModuleNotificationFunc>) -> Status;

    static RedisModule_CreateTimer:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  types: libc::c_longlong,
                  callback: Option<RedisModuleTimerProc>,
                  data: *mut libc::c_void) -> RedisModuleTimerID;

    static RedisModule_StopTimer:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  id: RedisModuleTimerID,
                  data: *mut *mut libc::c_void) -> Status;

    static RedisModule_GetTimerInfo:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  id: RedisModuleTimerID,
                  remaining: libc::uint64_t,
                  data: *mut *mut libc::c_void) -> Status;
}

///
///
///
pub mod call1 {
    pub fn call(
        ctx: *mut ::redis::api::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut ::redis::api::RedisModuleString,
    ) -> *mut ::redis::api::RedisModuleCallReply {
        unsafe { RedisModule_Call(ctx, cmdname, fmt, arg0) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Call:
        extern "C" fn(
            ctx: *mut ::redis::api::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut ::redis::api::RedisModuleString,
        ) -> *mut ::redis::api::RedisModuleCallReply;
    }
}

///
///
///
pub mod replicate1 {
    pub fn call(
        ctx: *mut ::redis::api::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut ::redis::api::RedisModuleString,
    ) -> ::redis::api::Status {
        unsafe { RedisModule_Replicate(ctx, cmdname, fmt, arg0) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Replicate:
        extern "C" fn(
            ctx: *mut ::redis::api::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut ::redis::api::RedisModuleString,
        ) -> ::redis::api::Status;
    }
}

///
///
///
pub mod call2 {
    pub fn call(
        ctx: *mut ::redis::api::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut ::redis::api::RedisModuleString,
        arg1: *mut ::redis::api::RedisModuleString,
    ) -> *mut ::redis::api::RedisModuleCallReply {
        unsafe { RedisModule_Call(ctx, cmdname, fmt, arg0, arg1) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Call:
        extern "C" fn(
            ctx: *mut ::redis::api::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut ::redis::api::RedisModuleString,
            arg1: *mut ::redis::api::RedisModuleString,
        ) -> *mut ::redis::api::RedisModuleCallReply;
    }
}

///
///
///
pub mod replicate2 {
    pub fn call(
        ctx: *mut ::redis::api::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut ::redis::api::RedisModuleString,
        arg1: *mut ::redis::api::RedisModuleString,
    ) -> ::redis::api::Status {
        unsafe { RedisModule_Replicate(ctx, cmdname, fmt, arg0, arg1) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Replicate:
        extern "C" fn(
            ctx: *mut ::redis::api::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut ::redis::api::RedisModuleString,
            arg1: *mut ::redis::api::RedisModuleString,
        ) -> ::redis::api::Status;
    }
}

///
///
///
pub mod call3 {
    pub fn call(
        ctx: *mut ::redis::api::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut ::redis::api::RedisModuleString,
        arg1: *mut ::redis::api::RedisModuleString,
        arg2: *mut ::redis::api::RedisModuleString,
    ) -> *mut ::redis::api::RedisModuleCallReply {
        unsafe { RedisModule_Call(ctx, cmdname, fmt, arg0, arg1, arg2) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Call:
        extern "C" fn(
            ctx: *mut ::redis::api::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut ::redis::api::RedisModuleString,
            arg1: *mut ::redis::api::RedisModuleString,
            arg2: *mut ::redis::api::RedisModuleString,
        ) -> *mut ::redis::api::RedisModuleCallReply;
    }
}

///
///
///
pub mod replicate3 {
    pub fn call(
        ctx: *mut ::redis::api::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut ::redis::api::RedisModuleString,
        arg1: *mut ::redis::api::RedisModuleString,
        arg2: *mut ::redis::api::RedisModuleString,
    ) -> ::redis::api::Status {
        unsafe { RedisModule_Replicate(ctx, cmdname, fmt, arg0, arg1, arg2) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Replicate:
        extern "C" fn(
            ctx: *mut ::redis::api::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut ::redis::api::RedisModuleString,
            arg1: *mut ::redis::api::RedisModuleString,
            arg2: *mut ::redis::api::RedisModuleString,
        ) -> ::redis::api::Status;
    }
}

///
///
///
pub mod call4 {
    pub fn call(
        ctx: *mut ::redis::api::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut ::redis::api::RedisModuleString,
        arg1: *mut ::redis::api::RedisModuleString,
        arg2: *mut ::redis::api::RedisModuleString,
        arg3: *mut ::redis::api::RedisModuleString,
    ) -> *mut ::redis::api::RedisModuleCallReply {
        unsafe { RedisModule_Call(ctx, cmdname, fmt, arg0, arg1, arg2, arg3) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Call:
        extern "C" fn(
            ctx: *mut ::redis::api::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut ::redis::api::RedisModuleString,
            arg1: *mut ::redis::api::RedisModuleString,
            arg2: *mut ::redis::api::RedisModuleString,
            arg3: *mut ::redis::api::RedisModuleString,
        ) -> *mut ::redis::api::RedisModuleCallReply;
    }
}

///
///
///
pub mod replicate4 {
    pub fn call(
        ctx: *mut ::redis::api::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut ::redis::api::RedisModuleString,
        arg1: *mut ::redis::api::RedisModuleString,
        arg2: *mut ::redis::api::RedisModuleString,
        arg3: *mut ::redis::api::RedisModuleString,
    ) -> ::redis::api::Status {
        unsafe { RedisModule_Replicate(ctx, cmdname, fmt, arg0, arg1, arg2, arg3) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Replicate:
        extern "C" fn(
            ctx: *mut ::redis::api::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut ::redis::api::RedisModuleString,
            arg1: *mut ::redis::api::RedisModuleString,
            arg2: *mut ::redis::api::RedisModuleString,
            arg3: *mut ::redis::api::RedisModuleString,
        ) -> ::redis::api::Status;
    }
}
