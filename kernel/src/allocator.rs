#[global_allocator]
static ALLOCATOR: Dummy = Dummy;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
pub struct Dummy;

pub static mut HEAP_START: usize = 0x0;
//pub static mut OFFSET: usize = 0x0;
//pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}

pub fn init_heap(offset: usize) {
    unsafe { HEAP_START += offset }
}