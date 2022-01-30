use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptStackFrame;
use crate::asm;
use crate::int::{InterruptIndex, PICS};
use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use crate::fifo::Fifo;

lazy_static! {
    pub static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,HandleControl::Ignore));
    pub static ref KEYBUF: Mutex<Fifo> = Mutex::new(Fifo::new(32));
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let scancode = asm::io_in8(0x60);
    KEYBUF.lock().put(scancode).unwrap();

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
