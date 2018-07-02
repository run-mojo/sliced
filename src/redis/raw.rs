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
pub enum ListMarker {
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


#[allow(non_snake_case)]
#[allow(unused_variables)]
//#[no_mangle]
extern "C" fn Empty_RDBLoad(rdb: *mut RedisModuleIO,
                            encver: libc::c_int) {}

#[allow(non_snake_case)]
#[allow(unused_variables)]
//#[no_mangle]
extern "C" fn Empty_RDBSave(rdb: *mut RedisModuleIO,
                            value: *mut u8) {}

#[allow(non_snake_case)]
#[allow(unused_variables)]
//#[no_mangle]
extern "C" fn Empty_AOFRewrite(rdb: *mut RedisModuleIO,
                               key: *mut RedisModuleString,
                               value: *mut u8) {}

#[allow(non_snake_case)]
#[allow(unused_variables)]
//#[no_mangle]
extern "C" fn Empty_MemUsage(rdb: *mut RedisModuleIO,
                             key: *mut RedisModuleString,
                             value: *mut u8) -> libc::size_t {
    return 0;
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
//#[no_mangle]
extern "C" fn Empty_Digest(digest: *mut RedisModuleDigest,
                           value: *mut u8) {}

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

///
/// RedisModule_CloseKey
///
pub fn close_key(kp: *mut RedisModuleKey) {
    unsafe { RedisModule_CloseKey(kp) }
}

///
/// RedisModule_CreateCommand
///
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

///
/// RedisModule_CreateString
///
pub fn create_string(
    ctx: *mut RedisModuleCtx,
    ptr: *const u8,
    len: libc::size_t,
) -> *mut RedisModuleString {
    unsafe { RedisModule_CreateString(ctx, ptr, len) }
}

pub fn free_string(ctx: *mut RedisModuleCtx, str: *mut RedisModuleString) {
    unsafe { RedisModule_FreeString(ctx, str) }
}

pub fn get_selected_db(ctx: *mut RedisModuleCtx) -> libc::c_int {
    unsafe { RedisModule_GetSelectedDb(ctx) }
}

pub fn log(ctx: *mut RedisModuleCtx, level: *const u8, fmt: *const u8) {
    unsafe { RedisModule_Log(ctx, level, fmt) }
}

pub fn open_key(
    ctx: *mut RedisModuleCtx,
    keyname: *mut RedisModuleString,
    mode: KeyMode,
) -> *mut RedisModuleKey {
    unsafe { RedisModule_OpenKey(ctx, keyname, mode) }
}

///
/// RedisModule_ReplyWithArray
///
pub fn reply_with_array(ctx: *mut RedisModuleCtx,
                        len: libc::c_long) -> Status {
    unsafe { RedisModule_ReplyWithArray(ctx, len) }
}

///
/// RedisModule_ReplyWithError
///
pub fn reply_with_error(ctx: *mut RedisModuleCtx,
                        err: *const u8) {
    unsafe { RedisModule_ReplyWithError(ctx, err) }
}

///
/// RedisModule_ReplyWithLongLong
///
pub fn reply_with_long_long(ctx: *mut RedisModuleCtx,
                            ll: libc::c_longlong) -> Status {
    unsafe { RedisModule_ReplyWithLongLong(ctx, ll) }
}

///
/// RedisModule_ReplyWithString
///
pub fn reply_with_string(
    ctx: *mut RedisModuleCtx,
    str: *mut RedisModuleString,
) -> Status {
    unsafe { RedisModule_ReplyWithString(ctx, str) }
}

/// RedisModule_SetExpire
///
/// -- Sets the expiry on a key.
/// -- Expire is in milliseconds.
pub fn set_expire(key: *mut RedisModuleKey,
                  expire: libc::c_longlong) -> Status {
    unsafe { RedisModule_SetExpire(key, expire) }
}

///
/// RedisModule_StringDMA
///
pub fn string_dma(
    key: *mut RedisModuleKey,
    len: *mut libc::size_t,
    mode: KeyMode,
) -> *const u8 {
    unsafe { RedisModule_StringDMA(key, len, mode) }
}

///
/// RedisModule_StringPtrLen
///
pub fn string_ptr_len(str: *mut RedisModuleString,
                      len: *mut libc::size_t) -> *const u8 {
    unsafe { RedisModule_StringPtrLen(str, len) }
}

///
/// RedisModule_StringSet
///
pub fn string_set(key: *mut RedisModuleKey,
                  str: *mut RedisModuleString) -> Status {
    unsafe { RedisModule_StringSet(key, str) }
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
pub fn subscribe_to_keyspace_events(ctx: *mut RedisModuleCtx, types: libc::c_int, callback: Option<RedisModuleNotificationFunc>) -> libc::c_int {
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
pub fn create_timer(ctx: *mut RedisModuleCtx, period: libc::c_longlong, callback: Option<RedisModuleTimerProc>, data: *mut u8) -> RedisModuleTimerID {
    unsafe { RedisModule_CreateTimer(ctx, period, callback, data) }
}

/// Stop a timer, returns REDISMODULE_OK if the timer was found, belonged to the
/// calling module, and was stoped, otherwise REDISMODULE_ERR is returned.
/// If not NULL, the data pointer is set to the value of the data argument when
/// the timer was created.
pub fn stop_timer(ctx: *mut RedisModuleCtx, id: RedisModuleTimerID, data: *mut *mut libc::c_void) -> Status {
    unsafe { RedisModule_StopTimer(ctx, id, data) }
}

/// Obtain information about a timer: its remaining time before firing
/// (in milliseconds), and the private data pointer associated with the timer.
/// If the timer specified does not exist or belongs to a different module
/// no information is returned and the function returns REDISMODULE_ERR, otherwise
/// REDISMODULE_OK is returned. The argumnets remaining or data can be NULL if
/// the caller does not need certain information.
pub fn get_timer_info(ctx: *mut RedisModuleCtx, id: RedisModuleTimerID, remaining: libc::uint64_t, data: *mut *mut libc::c_void) -> Status {
    unsafe { RedisModule_GetTimerInfo(ctx, id, remaining, data) }
}


///
/// Tap directly into Redis memory management
///
extern "C" {
    pub fn zmalloc(size: libc::size_t) -> *mut libc::c_void;
    pub fn zrealloc(p: *mut libc::c_void, size: libc::size_t) -> *mut libc::c_void;
    pub fn zfree(p: *mut libc::c_void);
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

    pub static RedisModule_CreateDataType:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  name: *const u8,
                  encver: libc::c_int,
                  typemethods: *const RedisModuleTypeMethods) -> Option<RedisModuleType>;

    pub static RedisModule_Milliseconds:
    extern "C" fn() -> *mut libc::c_longlong;

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
    extern "C" fn(ctx: *mut RedisModuleCtx, ptr: *const u8, len: libc::size_t)
                  -> *mut RedisModuleString;

    static RedisModule_FreeString:
    extern "C" fn(ctx: *mut RedisModuleCtx, str: *mut RedisModuleString);

    static RedisModule_GetSelectedDb: extern "C" fn(ctx: *mut RedisModuleCtx) -> libc::c_int;

    static RedisModule_Log:
    extern "C" fn(ctx: *mut RedisModuleCtx, level: *const u8, fmt: *const u8);

    static RedisModule_OpenKey:
    extern "C" fn(
        ctx: *mut RedisModuleCtx,
        keyname: *mut RedisModuleString,
        mode: KeyMode,
    ) -> *mut RedisModuleKey;

    static RedisModule_ReplyWithArray:
    extern "C" fn(ctx: *mut RedisModuleCtx, len: libc::c_long) -> Status;

    static RedisModule_ReplyWithError:
    extern "C" fn(ctx: *mut RedisModuleCtx, err: *const u8);

    static RedisModule_ReplyWithLongLong:
    extern "C" fn(ctx: *mut RedisModuleCtx, ll: libc::c_longlong) -> Status;

    static RedisModule_ReplyWithString:
    extern "C" fn(ctx: *mut RedisModuleCtx, str: *mut RedisModuleString) -> Status;

    static RedisModule_SetExpire:
    extern "C" fn(key: *mut RedisModuleKey, expire: libc::c_longlong) -> Status;

    static RedisModule_StringDMA:
    extern "C" fn(key: *mut RedisModuleKey, len: *mut libc::size_t, mode: KeyMode) -> *const u8;

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

    static RedisModule_SubscribeToKeyspaceEvents:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  types: libc::c_int,
                  callback: Option<RedisModuleNotificationFunc>) -> libc::c_int;

    static RedisModule_CreateTimer:
    extern "C" fn(ctx: *mut RedisModuleCtx,
                  types: libc::c_longlong,
                  callback: Option<RedisModuleTimerProc>,
                  data: *mut u8) -> RedisModuleTimerID;

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
    use redis::raw;

    pub fn call(
        ctx: *mut raw::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut raw::RedisModuleString,
    ) -> *mut raw::RedisModuleCallReply {
        unsafe { RedisModule_Call(ctx, cmdname, fmt, arg0) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Call:
        extern "C" fn(
            ctx: *mut raw::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut raw::RedisModuleString,
        ) -> *mut raw::RedisModuleCallReply;
    }
}

///
///
///
pub mod call2 {
    use redis::raw;

    pub fn call(
        ctx: *mut raw::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut raw::RedisModuleString,
        arg1: *mut raw::RedisModuleString,
    ) -> *mut raw::RedisModuleCallReply {
        unsafe { RedisModule_Call(ctx, cmdname, fmt, arg0, arg1) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Call:
        extern "C" fn(
            ctx: *mut raw::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut raw::RedisModuleString,
            arg1: *mut raw::RedisModuleString,
        ) -> *mut raw::RedisModuleCallReply;
    }
}

///
///
///
pub mod call3 {
    use redis::raw;

    pub fn call(
        ctx: *mut raw::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut raw::RedisModuleString,
        arg1: *mut raw::RedisModuleString,
        arg2: *mut raw::RedisModuleString,
    ) -> *mut raw::RedisModuleCallReply {
        unsafe { RedisModule_Call(ctx, cmdname, fmt, arg0, arg1, arg2) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Call:
        extern "C" fn(
            ctx: *mut raw::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut raw::RedisModuleString,
            arg1: *mut raw::RedisModuleString,
            arg2: *mut raw::RedisModuleString,
        ) -> *mut raw::RedisModuleCallReply;
    }
}

///
///
///
pub mod call4 {
    use redis::raw;

    pub fn call(
        ctx: *mut raw::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut raw::RedisModuleString,
        arg1: *mut raw::RedisModuleString,
        arg2: *mut raw::RedisModuleString,
        arg3: *mut raw::RedisModuleString,
    ) -> *mut raw::RedisModuleCallReply {
        unsafe { RedisModule_Call(ctx, cmdname, fmt, arg0, arg1, arg2, arg3) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Call:
        extern "C" fn(
            ctx: *mut raw::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut raw::RedisModuleString,
            arg1: *mut raw::RedisModuleString,
            arg2: *mut raw::RedisModuleString,
            arg3: *mut raw::RedisModuleString,
        ) -> *mut raw::RedisModuleCallReply;
    }
}
