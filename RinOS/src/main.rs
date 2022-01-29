#![feature(abi_x86_interrupt)]
#![no_std] // 不链接 Rust 标准库
#![no_main] // 禁用所有 Rust 层级的入口点

mod asm;
mod vga;
mod font;
mod int;
mod serial;
mod gdt;
mod fifo;

use core::panic::PanicInfo;
use pc_keyboard::DecodedKey;
use crate::asm::io_sti;
use crate::int::{KEYBUF, MOUSEBUF, KEYBOARD};
use crate::vga::{Color, SCREEN, WRITER};

#[no_mangle] // 不重整函数名
pub extern "C" fn _start() -> ! {
    // 因为编译器会寻找一个名为 `_start` 的函数，所以这个函数就是入口点
    // 默认命名为 `_start`

    //init
    int::init_idt();
    gdt::init_gdt();
    unsafe { int::PICS.lock().initialize(); }
    int::enable_mouse();
    io_sti();

    SCREEN.lock().init();
    WRITER.lock().set(Color::White, 8, 16);
    use core::fmt::Write;
    write!(WRITER.lock(), "Welcome to").unwrap(); //字符串会吞掉换行后面的字符
    WRITER.lock().set(Color::Black, 33, 33);
    write!(WRITER.lock(), "Rin OS.").unwrap();
    WRITER.lock().set(Color::White, 32, 32);
    write!(WRITER.lock(), "Rin OS.").unwrap();
    loop {
        asm::io_cli();
        if KEYBUF.lock().status() != 0 {
            let scancode = KEYBUF.lock().get().unwrap();
            asm::io_sti();
            let mut kbd = KEYBOARD.lock();
            if let Ok(Some(key_event)) = kbd.add_byte(scancode) {
                if let Some(key) = kbd.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(chr) => serial_print!("{}", chr),
                        DecodedKey::RawKey(key) => serial_print!("{:?}", key)
                    }
                }
            }
        } else if MOUSEBUF.lock().status() != 0 {
            let data = MOUSEBUF.lock().get().unwrap();
            asm::io_sti();
            serial_print!("{:x}", data);
        } else {
            asm::io_stihlt();
        }
        asm::io_hlt();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// 这个函数将在 panic 时被调用
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", _info);
    exit_qemu(QemuExitCode::Failed);
    loop {
        asm::io_hlt();
    }
}
