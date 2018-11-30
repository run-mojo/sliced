extern crate libc;

use crate::redis::redmod;

#[allow(unused_variables)]
pub struct DataTypes {
//    histogram: HistogramType
}

pub fn load(ctx: *mut redmod::RedisModuleCtx) -> redmod::Status {
    redmod::create_data_type(ctx,
                             format!("{}\0", "histogram").as_ptr(),
                             0,
                             Some(HistogramType::Histogram_RDBLoad),
                             Some(HistogramType::Histogram_RDBSave),
                             Some(HistogramType::Histogram_AOFRewrite),
                             Some(HistogramType::Histogram_MemUsage),
                             Some(HistogramType::Histogram_Digest),
                             Some(HistogramType::Histogram_Free));

    return redmod::Status::Ok;
}

pub struct HistogramType;

#[allow(unused_variables)]
#[allow(non_snake_case)]
impl HistogramType {
//    use redis::raw;
//    use libc;


    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_RDBLoad(rdb: *mut redmod::RedisModuleIO,
                                        encver: libc::c_int) {
//        log_debug!(self, "Histogram_RDBLoad");
        println!("Histogram_RDBLoad");
    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_RDBSave(rdb: *mut redmod::RedisModuleIO,
                                        value: *mut u8) {
//        log_debug!(self, "{} [began] args = {:?}", command, args);
        println!("Histogram_RDBSave");
    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_AOFRewrite(rdb: *mut redmod::RedisModuleIO,
                                           key: *mut redmod::RedisModuleString,
                                           value: *mut u8) {
        println!("Histogram_AOFRewrite");
    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_MemUsage(rdb: *mut redmod::RedisModuleIO,
                                         key: *mut redmod::RedisModuleString,
                                         value: *mut u8) -> libc::size_t {
        println!("Histogram_MemUsage");
        return 0;
    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_Digest(digest: *mut redmod::RedisModuleDigest,
                                       value: *mut u8) {
        println!("Histogram_Digest");
    }

    #[allow(non_snake_case)]
    #[allow(unused_variables)]
    #[no_mangle]
    pub extern "C" fn Histogram_Free(value: *mut u8) {
        println!("Histogram_Free");
//        unsafe { zfree(value as *mut libc::c_void); }
    }
}