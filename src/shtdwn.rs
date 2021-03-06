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
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum KIOOpResult {
    Success,
    ReadResultByte(u8),
    ReadResultWord(u16),
    ReadResultDWord(u32),
    ReadResultQWord(u64),
    Failure(String),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IOOpData {
    WriteByte(u8),
    WriteWord(u16),
    WriteDWord(u32),
    WriteQWord(u64),

    ReadByte(),
    ReadWord(),
    ReadDWord(),
    ReadQWord(),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IOOp {
    pub port: u16,
    pub data: IOOpData,
}

#[macro_export]
macro_rules! println {
    ($($tail:tt)*) => { writeln!(liboh::klog::KLog, $($tail)*).unwrap(); }
}

pub fn main_fn() {
    // "\ue0b0"
    println!("ohes is shutting down...");

    println!(" ---- qemu method ----");
    let _: KIOOpResult = liboh::service::request(
        "kio",
        IOOp {
            port: 0x604,
            data: IOOpData::WriteWord(0x2000),
        },
    );
    println!(" ---- bochs/old qemu method ----");
    let _: KIOOpResult = liboh::service::request(
        "kio",
        IOOp {
            port: 0xB004,
            data: IOOpData::WriteWord(0x2000),
        },
    );
    println!(" ---- vbox method ----");
    let _: KIOOpResult = liboh::service::request(
        "kio",
        IOOp {
            port: 0x4004,
            data: IOOpData::WriteWord(0x3400),
        },
    );
    println!(" AYYY, get a better vm will ya? i can't power off!")
}
main!(main_fn);
