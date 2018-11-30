#![feature(futures_api, await_macro, async_await)]

extern crate sliced;
extern crate time;

use sliced::mmap::*;
use sliced::redis::listpack::*;
use sliced::redis::rax::*;
use std::cmp::Ordering;
use std::fmt;

fn main() {
    let m = async || {
        "hi"
    };

    println!("hi");
}

