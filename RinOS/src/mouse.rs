use lazy_static::lazy_static;
use ps2_mouse::{Mouse, MouseState};
use x86_64::structures::idt::InterruptStackFrame;
use crate::{asm, SCREEN, serial_println};
use crate::int::{InterruptIndex, PICS};
use spin::Mutex;

lazy_static! {
    pub static ref MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());
}

pub extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let packet = asm::io_in8(0x60);
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
    if mouse_state.moved() {
        let dx = mouse_state.get_x();
        let dy = mouse_state.get_y();
        SCREEN.lock().hide_mouse_cursor();
        SCREEN.lock().set_mouse_pos(dx as isize, dy as isize);
        SCREEN.lock().update_mouse_cursor();
    }
}
