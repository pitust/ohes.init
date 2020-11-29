#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(box_syntax)]

use core::{alloc::Layout, fmt::Write};
use liboh::prelude::*;
extern crate rlibc;
extern crate alloc;
use postcard;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum KSvcResult {
    Success,
    Failure(String),
}

fn kdoapi_log(s: String) {
    let d = postcard::to_allocvec(&s).unwrap();
    liboh::syscall::sys_bindbuffer(d.as_slice());
    liboh::syscall::sys_send("log");
    let l = liboh::syscall::sys_getbufferlen();
    let a = unsafe { alloc::alloc::alloc(Layout::from_size_align(l as usize, 8).unwrap()) }; 
    let slc = unsafe { core::slice::from_raw_parts_mut(a, l as usize) };
    liboh::syscall::sys_readbuffer(slc);
    let sv: KSvcResult = postcard::from_bytes(slc).unwrap();
    writeln!(liboh::klog::KLog, "{:?}", sv);
}

fn main_fn() {
    let b = box 3;
    liboh::syscall::sys_klog("Hello, world!\n");
    writeln!(liboh::klog::KLog, "{}", b);
    kdoapi_log("The test!!!\n".to_string());
}
main!(main_fn);
