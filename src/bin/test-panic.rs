#![feature(panic_implementation)]
#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate mini_os;

use core::panic::PanicInfo;
use mini_os::exit_qemu;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    panic!();
}

#[cfg(not(test))]
#[panic_implementation]
#[no_mangle]
pub fn panic(_info: &PanicInfo) -> ! {
    serial_println!("ok");
    unsafe { exit_qemu(); }
    loop {}
}

//Bootimage Test
//The test runner of the bootimage tool can be invoked via bootimage test. It uses the following conventions:
//
//All executables starting with test- are treated as integration tests.
//Tests must print either ok or failed over the serial port. When printing failed they can print additional information such as a panic message (in the next lines).
//Tests are run with a timeout of 1 minute. If the test has not completed in time, it is reported as "timed out".
//The test-basic-boot and test-panic tests we created above begin with test- and follow the ok/failed conventions, so they should work with bootimage test: