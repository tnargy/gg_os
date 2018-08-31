#![feature(panic_implementation)] // required for defining the panic handler
#![feature(abi_x86_interrupt)] // required for defining the x86-interrupts
#![no_std] // don't link the Rust standard library
#![feature(const_fn)]

extern crate spin;
extern crate volatile;
#[macro_use]
extern crate lazy_static;
extern crate cpuio;
extern crate multiboot2;
extern crate pic8259_simple;
extern crate uart_16550;
extern crate x86_64;

use core::panic::PanicInfo;
use x86_64::structures::idt::{ExceptionStackFrame, InterruptDescriptorTable};

mod gdt;
//mod serial;
#[macro_use]
mod vga_buffer;
mod interrupts;
mod keyboard;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    gdt::init();
    init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    vga_buffer::clear_screen();
    println!("Booting Kernel...");

    // Get boot info from multiboot / GRUB
    let boot_info = unsafe { multiboot2::load(multiboot_information_address) };

    // Read Memory from BIOS
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    println!("Memory Areas:");
    for area in memory_map_tag.memory_areas() {
        println!(
            "    start: 0x{:x}, end: 0x{:x}, length: 0x{:x}",
            area.start_address(),
            area.end_address(),
            area.size()
        );
    }

    // Read Elf Sections from Kernel
    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("Elf-sections tag required");
    println!("Kernel Sections:");
    for section in elf_sections_tag.sections() {
        println!(
            "    addr: 0x{:x}, size: 0x{:x}, flags 0x{:x}",
            section.start_address(),
            section.size(),
            section.flags()
        );
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
        } else if input as usize == 0x0E {
            // TODO Backspace
        } else if input as usize == 0x01 {
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
    _error_code: u64,
) {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
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
