#![no_std]
#![no_main]
#![feature(panic_implementation)]

#[macro_use]
extern crate lazy_static;
extern crate volatile;
extern crate spin;

#[macro_use]
mod vga_buffer;

use core::panic::PanicInfo;


/// The eh_personality language item is used for implementing stack unwinding.
/// By default, Rust uses unwinding to run the destructors of all live
/// stack variables in case of a panic. This ensures that all used memory
/// is freed and allows the parent thread to catch the panic and continue
/// execution. Unwinding, however, is a complicated process and requires
/// some OS specific libraries (e.g. libunwind on Linux or structured exception handling on Windows),
/// so we don't want to use it for our operating system.

/// In a typical Rust binary that links the standard library,
/// execution starts in a C runtime library called crt0 (“C runtime zero”),
/// which sets up the environment for a C application.
/// This includes creating a stack and placing the arguments in the right registers.
/// The C runtime then invokes the entry point of the Rust runtime,
/// which is marked by the start language item. Rust only has a very minimal runtime,
/// which takes care of some small things such as setting up stack overflow guards
/// or printing a backtrace on panic. The runtime then finally calls the main function.

/// On Linux, the default entry point is called _start.
/// The linker just looks for a function with that name and sets
/// this function as entry point the executable. So to overwrite
/// the entry point, we define our own _start function:
static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    panic!("Some panic");
    println!("hello os {}", "!");
    loop {}
}

/// On Windows, the linker requires two entry points depending on
/// the used subsystem. For the CONSOLE subsystem, we need a function
/// called mainCRTStartup, which calls a function called main.
/// Like on Linux, we overwrite the entry points by defining
/// no_mangle functions:
#[no_mangle]
pub extern "C" fn mainCRTStartup() -> ! {
    main();
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}

/// macOS does not support statically linked binaries,
/// so we have to link the libSystem library. The entry point
/// is called main:
//#[no_mangle]
//pub extern "C" fn main()-> ! {
//    loop {
//
//    }
//}

/// This function is called on panic.
#[panic_implementation]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
