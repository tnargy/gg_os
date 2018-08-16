#![feature(panic_implementation)] // required for defining the panic handler
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

#[macro_use]
extern crate lazy_static;
extern crate bootloader_precompiled;
extern crate volatile;
extern crate spin;
use core::panic::PanicInfo;

#[macro_use]
mod vga_buffer;

/// This function is called on panic.
#[panic_implementation]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    //use core::fmt::Write;
    //vga_buffer::WRITER.lock().write_str("Hello again").unwrap();
    //write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}", 42, 1.337).unwrap();
    println!("Hello World{}", "!");
    panic!("Some panic message");
    
    loop {}
}
