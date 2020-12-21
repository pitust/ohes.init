#![feature(default_alloc_error_handler)]
#![feature(box_syntax)]
#![no_std]
#![no_main]
#![feature(alloc_prelude)]

use core::{alloc::Layout, fmt::Write};
use liboh::prelude::*;

pub use alloc::{
    borrow, boxed::Box, collections, collections::*, collections::*, fmt, format, prelude::v1::*,
    slice, string::*, vec, vec::Vec,
};
pub use core::sync::atomic::*;

extern crate alloc;
extern crate rlibc;
use postcard;

#[macro_export]
macro_rules! println {
    ($($tail:tt)*) => { writeln!(liboh::klog::KLog, $($tail)*).unwrap(); }
}
fn read_buf<'a, U: serde::Deserialize<'a> + Clone>() -> U {
    let l = liboh::syscall::sys_getbufferlen();
    let a = unsafe { alloc::alloc::alloc(Layout::from_size_align(l as usize, 8).unwrap()) };
    let slc = unsafe { core::slice::from_raw_parts_mut(a, l as usize) };
    liboh::syscall::sys_readbuffer(slc);
    let x: U = postcard::from_bytes::<'a, U>(slc).unwrap().clone();
    unsafe { alloc::alloc::dealloc(a, Layout::from_size_align(l as usize, 8).unwrap()) }
    x
}

pub fn main_fn() {
    // "\ue0b0"
    println!("[info] {}", read_buf::<Vec<String>>()[1]);
}
main!(main_fn);
