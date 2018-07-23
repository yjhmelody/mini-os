#![no_std]
#![cfg_attr(not(test), no_main)]
#![feature(panic_implementation)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![feature(abi_x86_interrupt)]

#[macro_use]
extern crate mini_os;
extern crate x86_64;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bootloader_precompiled;
#[macro_use]
extern crate bootloader;

use core::panic::PanicInfo;
use mini_os::*;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::ExceptionStackFrame;

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

//#[cfg(not(test))]
//#[no_mangle]
//pub extern "C" fn _start() -> ! {
//    println!("hello os {}", "!");
//
//    init_idt();
//    // invoke a breakpoint exception
//    x86_64::instructions::int3();
//    loop {}
//}
#[cfg(not(test))]
entry_point!(kernel_main);

#[cfg(not(test))]
#[no_mangle]
fn kernel_main(boot_info: &'static bootloader_precompiled::bootinfo::BootInfo) -> ! {
    println!("hello os {}", "!");
    gdt::init();
    init_idt();
    // invoke a breakpoint exception
//     x86_64::instructions::int3();

    // cause a double fault
//    unsafe {
//        *(0xdeadbeaf as *mut u64) = 42;
//    }
//
    // trigger a stack overflow
    fn stack_overflow() { stack_overflow(); };
    stack_overflow();


    println!("It did not crash!");
    loop {}
}

/// On Windows, the linker requires two entry points depending on
/// the used subsystem. For the CONSOLE subsystem, we need a function
/// called mainCRTStartup, which calls a function called main.
/// Like on Linux, we overwrite the entry points by defining
/// no_mangle functions:
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn mainCRTStartup() -> ! {
    main();
}


/// macOS does not support statically linked binaries,
/// so we have to link the libSystem library. The entry point
/// is called main:
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> ! {
    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_implementation]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// deal with double fault
extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut ExceptionStackFrame, _error_code: u64)
{
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}