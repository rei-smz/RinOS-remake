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

use core::panic::PanicInfo;
use ::vga::colors::Color16;
use pc_keyboard::DecodedKey;
use crate::asm::{io_hlt, io_sti, io_stihlt};
use crate::keyboard::{KEYBOARD, KEYBUF};
use crate::mouse::{MOUSE_CURSOR_WIDTH, MOUSE_CURSOR_HEIGHT, MOUSE_CURSOR};
use crate::vga::{VGA, SCREEN_WIDTH, SCREEN_HEIGHT};
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use crate::memory::BootInfoFrameAllocator;
extern crate alloc;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use core::borrow::BorrowMut;
use core::slice::SliceIndex;
use ::vga::writers::GraphicsWriter;
use lazy_static::lazy_static;
use crate::layer::{bg_layer_index, LAYERCTL, mouse_layer_index};

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

    // // allocate a number on the heap
    // let heap_value = Box::new(41);
    // serial_println!("heap_value at {:p}", heap_value);
    //
    // // create a dynamically sized vector
    // let mut vec = Vec::new();
    // for i in 0..500 {
    //     vec.push(i);
    // }
    // serial_println!("vec at {:p}", vec.as_slice());
    //
    // // create a reference counted vector -> will be freed when count reaches 0
    // let reference_counted = Rc::new(vec![1, 2, 3]);
    // let cloned_reference = reference_counted.clone();
    // serial_println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    // core::mem::drop(reference_counted);
    // serial_println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    //SCREEN.lock().init();
    // WRITER.lock().set(Color16::White, 8, 16);
    // use core::fmt::Write;
    // write!(WRITER.lock(), "Welcome to").unwrap(); //字符串会吞掉换行后面的字符
    // WRITER.lock().set(Color16::Black, 33, 33);
    // write!(WRITER.lock(), "Rin OS.").unwrap();
    // WRITER.lock().set(Color16::White, 32, 32);
    // write!(WRITER.lock(), "Rin OS.").unwrap();

    let mut mouse: Vec<Color16> = vec![Color16::Black; MOUSE_CURSOR_WIDTH * MOUSE_CURSOR_HEIGHT];
    let mut background: Vec<Color16> = vec![Color16::Black; SCREEN_WIDTH * SCREEN_HEIGHT];

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
    vga::init_screen(&mut background);
    LAYERCTL.lock().slide(*mouse_layer_index.lock(), (640 - 16) / 2, (480 -28 - 16) / 2);
    LAYERCTL.lock().up_down(*bg_layer_index.lock(), Some(0));
    LAYERCTL.lock().up_down(*mouse_layer_index.lock(), Some(1));

    loop {
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
