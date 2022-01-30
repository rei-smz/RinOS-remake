use x86_64::instructions::interrupts::{enable, disable, enable_and_hlt};
use x86_64::instructions::{hlt};
use x86_64::instructions::port::{Port, PortReadOnly};
use x86_64::registers::rflags;

pub fn io_hlt() {
    hlt();
}

pub fn io_cli() {
    disable();
}

pub fn io_sti() {
    enable();
}

pub fn io_stihlt() {
    enable_and_hlt();
}

pub fn io_out8(port: u16, data: u8) {
    unsafe {
        let mut p = Port::new(port);
        p.write(data);
    }
}

pub fn io_in8(port: u16) -> u8 {
    let ret: u8;
    unsafe {
        let mut p = PortReadOnly::new(port);
        ret = p.read();
    }
    ret
}

pub fn io_load_flags() -> rflags::RFlags {
    let ret = rflags::read();
    ret
}

pub fn io_store_flags(flags: rflags::RFlags) {
    unsafe {
        rflags::write(flags);
    }
}
