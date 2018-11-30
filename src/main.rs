#![feature(allocator_api)]
#![feature(async_await, await_macro, pin, arbitrary_self_types, futures_api)]

extern crate chrono;
extern crate env_logger;
//#[macro_use]
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
#[macro_use]
extern crate rand;
extern crate sliced;
extern crate spin;
extern crate tempdir;
extern crate time;
//extern crate hyper;

//use futures::{future, join, pending, Poll, poll, select, try_join};
//use futures::channel::oneshot;
//use futures::executor::block_on;
//use futures::Future;
//use futures::future::Map;
//use futures::future::poll_fn;
//use futures::prelude::*;
//use futures::sync::oneshot;
use rand::{Rng, thread_rng};
use sliced::alloc::*;
use sliced::mmap::*;
use sliced::mmap::MmapMut;
use sliced::redis::listpack::*;
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
use std::cell::{Cell, RefCell, UnsafeCell};
use tempdir::TempDir;

fn main() -> Result<(), StreamError> {
    let mut manager = StreamManager::new(
        SDS::new("mybucket"),
        std::path::Path::new("/Users/clay/.sliced"),
    )?;

    let mut stream: Rc<UnsafeCell<Stream>> = manager.create_stream(SDS::new("mystream"))?;

    let ss = unsafe { &mut *stream.get() };

//    let mut mut_stream = stream.as_ref();

    let mut lp = Listpack::new();
    lp.append(0); // MS
    lp.append(0); // Seq
    lp.append(10); // Offset
    lp.append(54); // Size
    lp.append(8); // Count

    let mut record_id = StreamID { ms: 0, seq: 0 };

    for _i in 0..10 {
        record_id = next_id(&record_id);
    }

    Ok(())
}

//fn main1() {
//    use std::sync::Arc;
//    use std::thread;
//
//    let five = Arc::new(5);
//
//    let mut v: Vec<std::thread::JoinHandle<()>> = vec![];
//
//    for _ in 0..10 {
//        let five = Arc::clone(&five);
//
//
//        v.push(thread::spawn(move || {
//            println!("{:?}", five);
////            println!("{}", five);
//            println!("Ref Count: {}", Arc::strong_count(&five));
//        }));
//    }
//
//    for a in v {
//        a.join();
//    }
//
//    println!("Ref Count: {}", Arc::strong_count(&five));
//}


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
