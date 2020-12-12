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
pub enum KSvcResult {
    Success,
    Failure(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum FSOp {
    Read,
    ReadDir,
    Stat,
}
#[derive(Serialize, Deserialize, Clone)]
pub enum FSResult {
    Text(Vec<u8>),
    Dirents(Vec<String>),
    Stats(u16),
    Failure(String),
}

// {
//  "_init": {
//    "run": ["fail", "-error", "_init is a meta-task that is launched by the init program itself"],
//  },
//  "target:boot": {
//   "trigger_after": "_init",
//   "run": ["kinfo", "-info", "Booted Oh Es!"],
//   "wants": ["fs", "net", "sircs"]
//  },
//  "fs.impl": {
//   "provides": "fs",
//   "run": ["fs", "-root", "kfs:/"]
//  },
// }

#[derive(Serialize, Deserialize, Debug)]
struct Node {
    run: Vec<String>,
    trigger_after: Option<String>,
    wants: Option<Vec<String>>,
    use_fs: Option<String>,
    with_fs: Option<String>,
    provides: Option<String>,
}
#[macro_export]
macro_rules! println {
        ($($tail:tt)*) => { writeln!(liboh::klog::KLog, $($tail)*).unwrap(); }
    }
fn write<T: serde::Serialize>(t: T) {
    let d = postcard::to_allocvec(&t).unwrap();
    liboh::syscall::sys_bindbuffer(d.as_slice());
}
fn ask_for_staf<'a, T: serde::Serialize, U: serde::Deserialize<'a> + Clone>(to: &str, t: T) -> U {
    let d = postcard::to_allocvec(&t).unwrap();
    liboh::syscall::sys_bindbuffer(d.as_slice());
    liboh::syscall::sys_send(to);
    let l = liboh::syscall::sys_getbufferlen();
    let a = unsafe { alloc::alloc::alloc(Layout::from_size_align(l as usize, 8).unwrap()) };
    let slc = unsafe { core::slice::from_raw_parts_mut(a, l as usize) };
    liboh::syscall::sys_readbuffer(slc);
    let x: U = postcard::from_bytes::<'a, U>(slc).unwrap().clone();
    unsafe { alloc::alloc::dealloc(a, Layout::from_size_align(l as usize, 8).unwrap()) }
    x
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

pub fn read_file(s: String) -> String {
    let t: FSResult = ask_for_staf("kfs", (FSOp::Read, s));
    let p = match t {
        FSResult::Text(p) => p,
        _ => unreachable!(),
    };
    String::from_utf8(p).unwrap()
}

fn exec(p: Vec<String>) {
    println!("{:?}", p);
}

fn we_did_task(
    rq: &mut VecDeque<String>,
    done: &mut BTreeSet<String>,
    r: &BTreeMap<String, Node>,
    t: String,
) {
    println!("Done {}", t);
    done.insert(t.clone());
    match &r[&t].provides {
        Some(p) => {
            done.insert(p.clone());
        }
        None => {}
    }
    for k in r {
        match &k.1.trigger_after {
            Some(p) => {
                if p == &t {
                    rq.push_back(k.0.clone())
                }
            }
            None => {}
        }
    }
}

fn do_task(
    r: String,
    rq: &mut VecDeque<String>,
    done: &mut BTreeSet<String>,
    pr: &BTreeMap<String, Node>,
) {
    if done.contains(&r) {
        return;
    }
    let mut can_do_now = true;
    if !pr.contains_key(&r) {
        for k in pr {
            match &k.1.provides {
                Some(u) => {
                    if u == &r {
                        rq.push_front(k.0.clone());
                        return;
                    }
                }
                None => {}
            }
        }
        println!("[ERR] Cannot find unknown task {}, skipping!", r);
        done.insert(r.clone());
        return;
    }
    match &pr[&r].wants {
        Some(p) => {
            for w in p {
                if done.contains(w) {
                    continue;
                }
                can_do_now = false;
                rq.push_back(w.clone());
            }
        }
        None => {}
    };
    if can_do_now {
        exec(pr[&r].run.clone());
        we_did_task(rq, done, pr, r);
    } else {
        rq.push_back(r);
    }
}

pub fn enforce<T>(s: serde_json::Result<T>) -> T {
    match s {
        Ok(p) => { p }
        Err(f) => {
            panic!("Failed reading init.rc:\n At {}:{} {}", f.line(), f.column(), f.to_string());
        }
    }
}

pub fn main_fn() {
    // "\ue0b0"
    println!(":: read init.rc...");
    // this is a hack to allow easier testing
    let txt = read_file("etc/init.rc".to_string());
    let p: BTreeMap<String, Node> = enforce(serde_json::from_str(&txt));
    let mut q = VecDeque::new();
    let mut done = BTreeSet::new();
    q.push_back("_init".to_string());
    we_did_task(&mut q, &mut done, &p, "_init".to_string());
    liboh::syscall::sys_listen("initd");
    loop {
        while q.len() != 0 {
            do_task(q.pop_front().unwrap(), &mut q, &mut done, &p);
        }
        liboh::syscall::sys_accept("initd");
        let x: String = read_buf();
        q.push_back(x);
        write(());
        liboh::syscall::sys_respond();
    }
}
main!(main_fn);
