#![feature(abi_x86_interrupt)]
#![feature(option_result_contains)]
#![no_std] // 不链接 Rust 标准库
#![no_main] // 禁用所有 Rust 层级的入口点

mod asm;
mod vga;
mod font;
mod int;
mod serial;
mod gdt;
mod fifo;
mod keyboard;
mod mouse;
mod memory;
mod layer;
mod window;
mod timer;

extern crate alloc;

use core::panic::PanicInfo;
use ::vga::colors::Color16;
use pc_keyboard::DecodedKey;
use crate::asm::{io_hlt, io_sti, io_stihlt};
use crate::keyboard::{KEYBOARD, KEYBUF};
use crate::mouse::{MOUSE_CURSOR_WIDTH, MOUSE_CURSOR_HEIGHT, MOUSE_CURSOR};
use crate::vga::{VGA, SCREEN_WIDTH, SCREEN_HEIGHT, LineWriter, update_mouse_cursor, boxfill};
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use crate::memory::BootInfoFrameAllocator;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc, format};
use core::borrow::BorrowMut;
use core::slice::SliceIndex;
use ::vga::writers::GraphicsWriter;
use lazy_static::lazy_static;
use ps2_mouse::MouseState;
use crate::layer::{bg_layer_index, LAYERCTL, mouse_layer_index, win_layer_index};
use spin::Mutex;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rin_os::allocator;

    //init
    int::init_idt();
    gdt::init_gdt();
    unsafe { int::PICS.lock().initialize(); }
    mouse::enable_mouse();
    io_sti();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset)};
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let mut mouse: Vec<Color16> = vec![Color16::Black; MOUSE_CURSOR_WIDTH * MOUSE_CURSOR_HEIGHT];
    let mut background: Vec<Color16> = vec![Color16::Black; SCREEN_WIDTH * SCREEN_HEIGHT];
    let mut window: Vec<Color16> = vec![Color16::Black; 160 * 52];

    //初始化鼠标指针
    for j in 0..MOUSE_CURSOR_HEIGHT {
        for i in 0..MOUSE_CURSOR_WIDTH {
            match MOUSE_CURSOR[j][i] {
                b'1' => mouse[j * MOUSE_CURSOR_WIDTH + i] = Color16::Black,
                b'0' => mouse[j * MOUSE_CURSOR_WIDTH + i] = Color16::White,
                _ => mouse[j * MOUSE_CURSOR_WIDTH + i] = Color16::Cyan
            }
        }
    }

    VGA.lock().set_mode();
    *bg_layer_index.lock() = LAYERCTL.lock().alloc().unwrap();
    *mouse_layer_index.lock() = LAYERCTL.lock().alloc().unwrap();
    *win_layer_index.lock() = LAYERCTL.lock().alloc().unwrap();
    LAYERCTL.lock().set_buf(
        *bg_layer_index.lock(),
        &mut background,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        None
    );
    LAYERCTL.lock().set_buf(
        *mouse_layer_index.lock(),
        &mut mouse,
        MOUSE_CURSOR_WIDTH,
        MOUSE_CURSOR_HEIGHT,
        Some(Color16::Cyan)
    );
    LAYERCTL.lock().set_buf(
        *win_layer_index.lock(),
        &mut window,
        160,
        52,
        None
    );
    vga::init_screen(&mut background);
    window::make_window(&mut window, 160, 52, "counter");
    // let mut writer = LineWriter::new(Color16::Black, 24, 28, 160, 68);
    // writer.write_str("Welcome to\nRinOS.", window.borrow_mut());
    LAYERCTL.lock().slide(*mouse_layer_index.lock(), (640 - 16) / 2, (480 -28 - 16) / 2);
    LAYERCTL.lock().slide(*win_layer_index.lock(), 80, 72);
    LAYERCTL.lock().up_down(*bg_layer_index.lock(), Some(0));
    LAYERCTL.lock().up_down(*win_layer_index.lock(), Some(1));
    LAYERCTL.lock().up_down(*mouse_layer_index.lock(), Some(2));

    let mut cnt = 0;
    loop {
        cnt += 1;
        if cnt % 10 == 0 {
            boxfill(window.borrow_mut(), Color16::LightGrey, 40, 28, 119, 43, 160);
            let mut writer = LineWriter::new(Color16::Black, 40, 28, 160, 52);
            writer.write_str(&format!("{:>010}", cnt / 10), window.borrow_mut());
            LAYERCTL.lock().refresh(*win_layer_index.lock(), 40, 28, 120, 44);
        }
        asm::io_cli();
        if KEYBUF.lock().status() != 0 {
            let scancode = KEYBUF.lock().get().unwrap();
            io_sti();
            let mut kbd = KEYBOARD.lock();
            if let Ok(Some(key_event)) = kbd.add_byte(scancode) {
                if let Some(key) = kbd.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(chr) => serial_println!("[KEYBUF]{}", chr),
                        DecodedKey::RawKey(key) => serial_println!("[KEYBUF]{:?}", key)
                    }
                }
            }
        } else {
            io_stihlt();
            //io_sti();
        }
        io_hlt();
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
