#![feature(panic_implementation)] // required for defining the panic handler
#![feature(abi_x86_interrupt)] // required for defining the x86-interrupts
#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate gg_os;
extern crate x86_64;
#[macro_use]
extern crate lazy_static;

use core::panic::PanicInfo;
use x86_64::structures::idt::{ExceptionStackFrame, InterruptDescriptorTable};

#[cfg(not(test))] // only compile when test flag is not set
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    gg_os::gdt::init();
    init_idt();
    unsafe { gg_os::interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    x86_64::instructions::int3();
    
    println!("It did not crash!");
    loop {}
}

#[cfg(not(test))] // only compile when test flag is not set
#[panic_implementation]
#[no_mangle]
/// This function is called on panic.
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

/// Create Interrupt Description Table
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gg_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        let timer_interrupt_id = usize::from(gg_os::interrupts::TIMER_INTERRUPT_ID);
        idt[timer_interrupt_id].set_handler_fn(timer_interrupt_handler);
            
        idt
    };
}

/// Load the IDT onto CPU
pub fn init_idt() {
    IDT.load();
}

/// Create Exception Breakpoint handler
extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// Create Timer Interrupt handler
extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: &mut ExceptionStackFrame)
{
    // PIC expects an explicit "end of interrupt" (EOI) signal
    unsafe { gg_os::interrupts::PICS.lock().notify_end_of_interrupt(
        gg_os::interrupts::TIMER_INTERRUPT_ID) }
}

/// Create Double Fault handler
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}
