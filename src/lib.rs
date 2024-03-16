use std::mem::MaybeUninit;
use std::time::Instant;

pub static mut START_TIME: MaybeUninit<Instant> = MaybeUninit::uninit();

pub mod cli;
pub mod console;
pub mod events;
pub mod fs;
pub mod module;
pub mod os;
pub mod path;
pub mod process;
pub mod vm;
