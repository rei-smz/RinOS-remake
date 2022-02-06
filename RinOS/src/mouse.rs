use lazy_static::lazy_static;
use ps2_mouse::{Mouse, MouseState};
use x86_64::structures::idt::InterruptStackFrame;
use crate::{asm, serial_println};
use crate::int::{InterruptIndex, PICS};
use spin::Mutex;
use crate::layer::{bg_layer_index, mouse_layer_index};
use crate::vga::{hide_mouse_cursor, update_mouse_cursor};

pub const MOUSE_CURSOR_WIDTH: usize = 16;
pub const MOUSE_CURSOR_HEIGHT: usize = 16;
pub const MOUSE_CURSOR: [[u8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] = [
    *b"111.............",
    *b"100111..........",
    *b"100000111.......",
    *b".10000000111....",
    *b".1000000000011..",
    *b".10000001111111.",
    *b"..1000001.......",
    *b"..10000001......",
    *b"..100110001.....",
    *b"...101.10001....",
    *b"...101..10001...",
    *b"...101...10001..",
    *b"....11....10001.",
    *b"....11.....10001",
    *b".....1......1001",
    *b".............111"
];

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
        hide_mouse_cursor(*bg_layer_index.lock());
        update_mouse_cursor(*mouse_layer_index.lock(), dx as isize, -dy as isize);
    }
}
