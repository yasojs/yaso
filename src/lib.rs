use std::mem::MaybeUninit;
use std::time::Instant;

pub static mut STARTING_TIME: MaybeUninit<Instant> = MaybeUninit::uninit();

pub mod cli;
pub mod console;
pub mod fs;
pub mod module;
pub mod os;
pub mod path;
pub mod process;
pub mod vm;
