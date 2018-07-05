extern crate libc;

//pub mod types;

pub struct DataTypes {
    histogram: HistogramType
}

use redis::api;

pub fn create_redis_types(ctx: *mut api::RedisModuleCtx) -> api::Status {
    api::create_data_type(ctx,
                          format!("{}\0", "histogram").as_ptr(),
                          0,
                          Some(HistogramType::Histogram_RDBLoad),
                          Some(HistogramType::Histogram_RDBSave),
                          Some(HistogramType::Histogram_AOFRewrite),
                          Some(HistogramType::Histogram_MemUsage),
                          Some(HistogramType::Histogram_Digest),
                          Some(HistogramType::Histogram_Free));

    return api::Status::Ok
}

pub struct HistogramType;

#[allow(non_snake_case)]
impl HistogramType {
//    use redis::raw;
//    use libc;


    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_RDBLoad(rdb: *mut api::RedisModuleIO,
                                        encver: libc::c_int) {
//        log_debug!(self, "Histogram_RDBLoad");
    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_RDBSave(rdb: *mut api::RedisModuleIO,
                                        value: *mut u8) {
//        log_debug!(self, "{} [began] args = {:?}", command, args);
    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_AOFRewrite(rdb: *mut api::RedisModuleIO,
                                           key: *mut api::RedisModuleString,
                                           value: *mut u8) {}

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_MemUsage(rdb: *mut api::RedisModuleIO,
                                         key: *mut api::RedisModuleString,
                                         value: *mut u8) -> libc::size_t {
        return 0;
    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_Digest(digest: *mut api::RedisModuleDigest,
                                       value: *mut u8) {

    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_Free(value: *mut u8) {
//        unsafe { zfree(value as *mut libc::c_void); }
    }
}