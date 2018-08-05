
extern crate sliced;
extern crate spin;
extern crate tempdir;
#[cfg(windows)]
extern crate winapi;

use sliced::mmap::{Mmap, MmapMut, MmapOptions};
use std::fs::OpenOptions;
use std::io::{Read, Write};
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;
use std::sync::Arc;
use std::thread;
#[cfg(windows)]
use winapi::um::winnt::GENERIC_ALL;


//#[test]
//fn append() {
//    let expected_len = 128;
//    let tempdir = tempdir::TempDir::new("mmap").unwrap();
//    let path = tempdir.path().join("mmap");
//
//    let mut file = OpenOptions::new()
//        .read(true)
//        .write(true)
//        .create(true)
//        .open(&path)
//        .unwrap();
//
//    let mut aof = AOF::new(&mut file).unwrap();
//    aof.append();
//
//    let mut p = 2_usize;
//    for _ in 0..20 {
//        println!("{}", p.next_power_of_two());
//        p = p + 1;
//    }
//
//
////        file.set_len(expected_len as u64).unwrap();
////
////        let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };
////        let len = mmap.len();
////        assert_eq!(expected_len, len);
////
////        let zeros = vec![0; len];
////        let incr: Vec<u8> = (0..len as u8).collect();
////
////        // check that the mmap is empty
////        assert_eq!(&zeros[..], &mmap[..]);
////
////        // write values into the mmap
////        (&mut mmap[..]).write_all(&incr[..]).unwrap();
////
////        // read values back
////        assert_eq!(&incr[..], &mmap[..]);
//}
//
//#[test]
//fn pow() {
//    let mut p = sliced::page_size::get();
//    for _ in 0..20 {
//        p = p.next_power_of_two();
//        println!("{}", p);
//        p = p + 1;
//    }
//}