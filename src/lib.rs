#![feature(panic_implementation)] // required for defining the panic handler
#![feature(const_fn)]
#![no_std]

extern crate bootloader_precompiled;
extern crate spin;
extern crate volatile;
#[macro_use]
extern crate lazy_static;
extern crate cpuio;
extern crate pic8259_simple;
extern crate uart_16550;
extern crate x86_64;

pub mod gdt;
pub mod serial;
#[macro_use]
pub mod vga_buffer;
pub mod interrupts;
pub mod keyboard;

use core::panic::PanicInfo;

pub unsafe fn exit_qemu() {
    use x86_64::instructions::port::Port;

    let mut port = Port::<u32>::new(0xf4);
    port.write(0);
}

#[panic_implementation]
#[no_mangle]
/// This function is called on panic.
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
