#![feature(panic_implementation)] // required for defining the panic handler
#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate mini_os;

use mini_os::*;

use core::panic::PanicInfo;

/// This function is the entry point, since the linker looks for a function
/// named `_start` by default.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("hello os {}", "!");
    serial_println!("ok");
    unsafe { exit_qemu(); }
    loop {}
}

#[cfg(not(test))]
#[panic_implementation]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    serial_println!("failed");
    serial_println!("{}", info);
    unsafe { exit_qemu(); }
    loop {}
}