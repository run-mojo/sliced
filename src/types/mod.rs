extern crate libc;

//pub mod types;

use redis::raw;

pub fn create_redis_types(ctx: *mut raw::RedisModuleCtx) -> raw::Status {
    raw::create_data_type(ctx,
                          format!("{}\0", "histogram").as_ptr(),
                          0,
                          Some(HistogramType::Histogram_RDBLoad),
                          Some(HistogramType::Histogram_RDBSave),
                          Some(HistogramType::Histogram_AOFRewrite),
                          Some(HistogramType::Histogram_MemUsage),
                          Some(HistogramType::Histogram_Digest),
                          Some(HistogramType::Histogram_Free));

    return raw::Status::Ok
}

pub struct HistogramType;

#[allow(non_snake_case)]
impl HistogramType {
//    use redis::raw;
//    use libc;


    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_RDBLoad(rdb: *mut raw::RedisModuleIO,
                                        encver: libc::c_int) {
//        log_debug!(self, "Histogram_RDBLoad");
    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_RDBSave(rdb: *mut raw::RedisModuleIO,
                                        value: *mut u8) {
//        log_debug!(self, "{} [began] args = {:?}", command, args);
    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_AOFRewrite(rdb: *mut raw::RedisModuleIO,
                                           key: *mut raw::RedisModuleString,
                                           value: *mut u8) {}

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_MemUsage(rdb: *mut raw::RedisModuleIO,
                                         key: *mut raw::RedisModuleString,
                                         value: *mut u8) -> libc::size_t {
        return 0;
    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_Digest(digest: *mut raw::RedisModuleDigest,
                                       value: *mut u8) {

    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_Free(value: *mut u8) {
//        unsafe { zfree(value as *mut libc::c_void); }
    }
}