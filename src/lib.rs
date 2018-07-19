#![no_std]
#![feature(panic_implementation)]

#[cfg(test)]
extern crate std;
#[cfg(test)]
extern crate array_init;
#[macro_use]
extern crate lazy_static;
extern crate volatile;
extern crate spin;
extern crate uart_16550;
extern crate x86_64;

#[macro_use]
pub mod vga_buffer;
#[macro_use]
pub mod serial;

pub unsafe fn exit_qemu() {
    use x86_64::instructions::port::Port;
    // iobase=0xf4,iosize=0x04
    let mut port = Port::<u32>::new(0xf4);
    // As value we write a zero, which causes QEMU to exit with exit status (0 << 1) | 1 = 1.
    port.write(0);
}
