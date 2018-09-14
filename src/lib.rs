#![feature(panic_implementation)] // required for defining the panic handler
#![feature(abi_x86_interrupt)] // required for defining the x86-interrupts
#![feature(const_fn)]
#![feature(ptr_internals)]
#![feature(alloc, allocator_api, alloc_error_handler)]
#![no_std] // don't link the Rust standard library

extern crate spin;
extern crate volatile;
#[macro_use]
extern crate lazy_static;
extern crate cpuio;
extern crate multiboot2;
extern crate pic8259_simple;
extern crate uart_16550;
extern crate x86_64;
#[macro_use]
extern crate bitflags;
extern crate alloc;
#[macro_use]
extern crate once;
extern crate rlibc;
extern crate linked_list_allocator;

use core::panic::PanicInfo;
use x86_64::structures::idt::{ExceptionStackFrame, InterruptDescriptorTable};
use linked_list_allocator::LockedHeap;

mod gdt;
#[macro_use]
mod vga_buffer;
mod interrupts;
mod keyboard;
#[macro_use]
mod serial;
mod memory;

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    gdt::init();
    init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    vga_buffer::clear_screen();

    // Get boot info from multiboot / GRUB
    let boot_info = unsafe { multiboot2::load(multiboot_information_address) };

    enable_nxe_bit();
    enable_write_protect_bit();

    // setup guard page and map the heap pages
    memory::init(&boot_info);

    // init the heap allocator
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_START + HEAP_SIZE);
    }
    
    use alloc::boxed::Box;
    let mut heap_test = Box::new(42);
    *heap_test -= 15;
    let heap_test2 = Box::new("hello");
    println!("{:?} {:?}", heap_test, heap_test2);

    use alloc::*;
    let mut vec_test = vec![1,2,3,4,5,6,7];
    vec_test[3] = 42;
    for i in &vec_test {
        print!("{} ", i);
    }
    println!();

    for i in 0..10000 {
        format!("Some String");
    }
    
    println!("READY!");

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
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        let timer_interrupt_id = usize::from(interrupts::TIMER_INTERRUPT_ID);
        let keyboard_interrupt_id = usize::from(interrupts::KEYBOARD_INTERRUPT_ID);

        idt[timer_interrupt_id].set_handler_fn(timer_interrupt_handler);
        idt[keyboard_interrupt_id].set_handler_fn(keyboard_interrupt_handler);

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
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut ExceptionStackFrame) {
    // PIC expects an explicit "end of interrupt" (EOI) signal
    unsafe {
        interrupts::PICS
            .lock()
            .notify_end_of_interrupt(interrupts::TIMER_INTERRUPT_ID)
    }
}

/// Create Keyboard Interrupt handler
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut ExceptionStackFrame) {
    if let Some(input) = keyboard::read_char() {
        if input == '\r' {
            println!("");
        } else if input == '\t' {
            print!("    ");
        } else if input as usize == 0 {
            vga_buffer::backspace();
        } else if input as usize == 27 {
            // TODO ESC
        } else {
            print!("{}", input);
        }
    }

    // PIC expects an explicit "end of interrupt" (EOI) signal
    unsafe {
        interrupts::PICS
            .lock()
            .notify_end_of_interrupt(interrupts::KEYBOARD_INTERRUPT_ID)
    }
}

/// Create Double Fault handler
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut ExceptionStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: DOUBLE FAULT {}\n{:#?}", error_code, stack_frame);
    loop {}
}

fn enable_nxe_bit() {
    use x86_64::registers::model_specific::*;
    unsafe {
        Efer::write(Efer::read() | EferFlags::NO_EXECUTE_ENABLE);
    }
}

fn enable_write_protect_bit() {
    use x86_64::registers::control::*;
    unsafe {
        Cr0::write(Cr0::read() | Cr0Flags::WRITE_PROTECT);
    }
}

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

#[alloc_error_handler]
#[no_mangle]
pub fn oom(layout: core::alloc::Layout) -> ! {
    panic!("Out of memory: {:?}", layout);
}
