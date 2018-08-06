#![feature(global_allocator, allocator_api, heap_api)]

extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate rand;
extern crate sliced;
extern crate spin;
extern crate tempdir;
extern crate time;
//extern crate futures;
//extern crate tokio;
//extern crate tokio_codec;
//extern crate tokio_current_thread;
//extern crate tokio_executor;
//extern crate tokio_fs;
//extern crate tokio_io;
//extern crate tokio_reactor;
//extern crate tokio_threadpool;
//extern crate tokio_timer;
extern crate libc;

//use futures::Future;
//use futures::future::Map;
//use futures::future::poll_fn;
//use futures::prelude::*;
//use futures::sync::oneshot;
use rand::{Rng, thread_rng};
use sliced::mmap::*;
use sliced::mmap::MmapMut;
use sliced::page_size;
use sliced::redis::listpack::*;
use sliced::alloc::*;
use sliced::redis::rax::*;
use sliced::redis::sds::*;
use sliced::stream::*;
use sliced::stream::id::*;

use spin::RwLock;
use std::cmp::Ordering;
use std::fmt;
use std::fs::File;
use std::io::{Read, SeekFrom};
use std::io::Error as IoError;
use std::io::Result as IoResult;
use std::mem;
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic;
use std::sync::atomic::AtomicUsize;
use std::time::{Duration, Instant};
use tempdir::TempDir;
//use tokio_fs::*;
//use tokio_io::io;
//use tokio_threadpool::*;


//#[global_allocator]
//static GLOBAL: sliced::alloc::RedisAllocator = sliced::alloc::RedisAllocator;


fn main() {
    let mut manager = StreamManager::new(
        SDS::new("mybucket"),
        std::path::Path::new("/Users/clay/sliced")
    );
    let mut size: usize = 1024 * 2;
    println!("PageSize: {}", sliced::page_size::get());
//    println!("RedisModule_Alloc -> {}", sliced::redis::api::RedisModule_Alloc);

    let mut record_id = StreamID { ms: 0, seq: 0 };

    for i in 0..10 {
//        println!("{}", size.next_power_of_two());
//        size = size + 1;
//        size = size.next_power_of_two();

        record_id = next_id(&record_id);
//        let record = Record {};
//        s.append(&mut record_id, &record);
    }
}

fn main1() {
    use std::sync::Arc;
    use std::thread;

    let five = Arc::new(5);

    let mut v:Vec<std::thread::JoinHandle<()>> = vec![];

    for _ in 0..10 {
        let five = Arc::clone(&five);


        v.push(thread::spawn(move || {
            println!("{:?}", five);
//            println!("{}", five);
            println!("Ref Count: {}", Arc::strong_count(&five));
        }));
    }

    for a in v {
        a.join();
    }

    println!("Ref Count: {}", Arc::strong_count(&five));
}


//fn main2() {
//    let dir = TempDir::new("tokio-fs-tests").unwrap();
//    let file_path = dir.path().join("seek.txt");
//
//    let pool = Builder::new().pool_size(2).max_blocking(2).build();
//    let (tx, rx) = oneshot::channel();
//
//    pool.spawn(
//        OpenOptions::new()
//            .create(true)
//            .read(true)
//            .write(true)
//            .open(file_path)
//            .and_then(|file| {
//                println!("opened file");
//                Ok(file)
//            })
//            .and_then(|file| {
//                println!("writing...");
//                io::write_all(file, "Hello, world!")
//            })
//            .and_then(|(file, _)| {
//                println!("seeking...");
//                file.seek(SeekFrom::End(-6))
//            })
//            .and_then(|(file, _)| {
//                println!("reading...");
//                io::read_exact(file, vec![0; 5])
//            })
//            .and_then(|(file, buf)| {
//                assert_eq!(buf, b"world");
//                file.seek(SeekFrom::Start(0))
//            })
//            .and_then(|(file, _)| io::read_exact(file, vec![0; 5]))
//            .and_then(|(_, buf)| {
//                assert_eq!(buf, b"Hello");
//                Ok(())
//            })
//            .then(|r| {
//                match r {
//                    Ok(rr) => {
//                        let _ = r.unwrap();
//                        tx.send(())
//                    }
//                    Err(e) => {
//                        match e.kind() {
//                            std::io::ErrorKind::NotFound => {
//                                println!("not found")
//                            }
//                            _ => {
//                                println!("something else")
//                            }
//                        }
//                        println!("Error");
//                        println!("{}", e);
//                        tx.send(())
//                    }
//                }
//            }),
//    );
//
//    match rx.wait() {
//        Ok(_) => {
//            println!("OK!")
//        }
//        Err(e) => {
//            println!("{}", e)
//        }
//    }
//    // rx.and_then(|r| {
//    //     println!("ending...");
//    //     Ok(())
//    // }).wait().unwrap();
//
//    // rx.wait().unwrap();
//}
