#![no_std]
#![feature(global_asm)]
#![feature(asm)]
//#![feature(renamed_spin_loop)]

//alloc test
#![feature(alloc_error_handler)]

#![feature(slice_fill)]

#[macro_use]
extern crate alloc;

#[macro_use]

mod io;
mod logger;

mod init;
mod lang_items;
mod sbi;

mod backtrace;

mod interrupt;
mod context;
mod timer;

mod cpu;
mod uart;
mod plic;

//mod fs;
mod consts;

mod fatfs;

mod net;
