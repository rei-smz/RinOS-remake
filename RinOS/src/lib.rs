#![no_std]
#![feature(alloc_error_handler)]
extern crate alloc;
pub mod allocator;
use linked_list_allocator::LockedHeap;
use core::panic;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
