use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{asm, Color, serial_print, serial_println};
use crate::vga::{LineWriter, Screen};
use core::fmt::Write;
use lazy_static::lazy_static;
use crate::gdt;
use pic8259::ChainedPics;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use crate::fifo::Fifo;
use ps2_mouse::{Mouse, MouseState};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
    Mouse = PIC_1_OFFSET + 12
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Mouse.as_usize()].set_handler_fn(mouse_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_print!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    serial_print!(".");

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

lazy_static! {
    pub static ref KEYBUF: Mutex<Fifo> = Mutex::new(Fifo::new(32));
    pub static ref MOUSEBUF: Mutex<Fifo> = Mutex::new(Fifo::new(128));
}

lazy_static! {
    pub static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,HandleControl::Ignore));
}

lazy_static! {
    pub static ref MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let scancode = asm::io_in8(0x60);
    KEYBUF.lock().put(scancode).unwrap();

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let packet = asm::io_in8(0x60);
    MOUSEBUF.lock().put(packet).unwrap();
    MOUSE.lock().process_packet(packet);

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Mouse.as_u8());
    }
}

pub fn enable_mouse() {
    MOUSE.lock().init().unwrap();
    MOUSE.lock().set_on_complete(on_mouse_complete);
}

fn on_mouse_complete(mouse_state: MouseState) {
    serial_println!("\n{:?}", mouse_state);
}
