use crate::redis::redmod;
use libc;
use super::*;

static mut STREAM_TYPE: usize = 0;

/// Called when Redis loads the module.
pub fn load(ctx: *mut redmod::RedisModuleCtx) -> redmod::Status {
    // Create Stream data type.
    redmod::create_data_type(ctx,
                             format!("{}\0", "mo.stream").as_ptr(),
                             0,
                             Some(Sliced_Type_Stream_RDBLoad),
                             Some(Sliced_Type_Stream_RDBSave),
                             Some(Sliced_Type_Stream_AOFRewrite),
                             Some(Sliced_Type_Stream_MemUsage),
                             Some(Sliced_Type_Stream_Digest),
                             Some(Sliced_Type_Stream_Free));

    return redmod::Status::Ok;
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn Sliced_Type_Stream_RDBLoad(rdb: *mut redmod::RedisModuleIO,
                                             encver: libc::c_int) {
//        log_debug!(self, "Histogram_RDBLoad");
    println!("slice/d Stream RDBLoad");
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn Sliced_Type_Stream_RDBSave(
    rdb: *mut redmod::RedisModuleIO,
    value: *mut u8) {
//        log_debug!(self, "{} [began] args = {:?}", command, args);
    println!("slice/d Stream RDBSave");
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn Sliced_Type_Stream_AOFRewrite(rdb: *mut redmod::RedisModuleIO,
                                                key: *mut redmod::RedisModuleString,
                                                value: *mut u8) {
    unsafe {
        let mut stream: *mut Stream = value as *mut _ as *mut Stream;
    }
    // Write to global AOF + Propagate
    // MO.X CREATE mystream id 10 seg 64mb pack 64kb
    // MO.X GROUP mystream mygroup
    // MO.X DELGROUP mystream mygroup
    // MO.X CONSUMER
    // MO.X DELCONSUMER
    // MO.X FAILED mystream mygroup id 10
    // MO.X SEG mystream 10
    // MO.X SEGMERGE mystream 10 11
    // MO.X SEGDEL mystream 10
    // MO.X DEL mystream
    // MO.X SEGMENT CREATE 10 10000003832-10
    // MO.XADD
    // MO.X SEGMENT FOLD 10000002340-10
    // MO.X UPLOAD 10 10000002340-10
    // MO.X UPLOADED 10 10000002340-10

    // Rewrite AOF
    // Save stream

    // MO.X SEGMENT ADD mystream 10 start 64mb max 64mb pack 64kb

    // MO.STREAM REMOVE SEGMENT mystream 10

    // MO.XADD
    // MO.XREADGROUP
    // MO.XREAD
    // MO.XRANGE
    // MO.XTRIM
    // MO.XDEL

    println!("slice/d Stream AOFRewrite");
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn Sliced_Type_Stream_MemUsage(rdb: *mut redmod::RedisModuleIO,
                                              key: *mut redmod::RedisModuleString,
                                              value: *mut u8) -> libc::size_t {
    println!("slice/d Stream MemUsage");
    return 0;
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn Sliced_Type_Stream_Digest(digest: *mut redmod::RedisModuleDigest,
                                            value: *mut u8) {
    println!("slice/d Stream Digest");
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn Sliced_Type_Stream_Free(value: *mut u8) {
    println!("slice/d Stream Free");
//        unsafe { zfree(value as *mut libc::c_void); }
}