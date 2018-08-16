#![feature(panic_implementation)] // required for defining the panic handler
#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[cfg(test)]
extern crate std;

#[cfg(test)]
extern crate array_init;

#[macro_use]
extern crate lazy_static;
extern crate bootloader_precompiled;
extern crate volatile;
extern crate spin;
use core::panic::PanicInfo;

#[macro_use]
mod vga_buffer;

#[cfg(not(test))]  // only compile when test flag is not set
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
        
    loop {}
}

#[cfg(not(test))]  // only compile when test flag is not set
#[panic_implementation]
#[no_mangle]
/// This function is called on panic.
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
