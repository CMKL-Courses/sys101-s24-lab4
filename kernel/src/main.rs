#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

mod screen;

use core::fmt::Write;
use core::slice;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use bootloader_api::config::Mapping::Dynamic;
use bootloader_api::info::MemoryRegionKind;
use kernel::{HandlerTable, serial};
use pc_keyboard::DecodedKey;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::Size4KiB;
use x86_64::VirtAddr;
use crate::screen::{Writer, screenwriter};

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Dynamic); // obtain physical memory offset
    config.kernel_stack_size = 256 * 1024; // 256 KiB kernel stack size
    config
};
entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    writeln!(serial(), "Entered kernel with boot info: {boot_info:?}").unwrap();
    writeln!(serial(), "Frame Buffer: {:p}", boot_info.framebuffer.as_ref().unwrap().buffer()).unwrap();

    let frame_info = boot_info.framebuffer.as_ref().unwrap().info();
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    screen::init(framebuffer);
    for x in 0..frame_info.width {
        screenwriter().draw_pixel(x, frame_info.height-15, 0xff, 0, 0);
        screenwriter().draw_pixel(x, frame_info.height-10, 0, 0xff, 0);
        screenwriter().draw_pixel(x, frame_info.height-5, 0, 0, 0xff);
    }

    for r in boot_info.memory_regions.iter() {
        writeln!(Writer, "{:?} {:?} {:?} {}", r, r.start as *mut u8, r.end as *mut usize, r.end-r.start).unwrap();
    }

    let usable_region = boot_info.memory_regions.iter().filter(|x|x.kind == MemoryRegionKind::Usable).last().unwrap();
    writeln!(Writer, "{usable_region:?}").unwrap();

    let physical_offset = boot_info.physical_memory_offset.into_option().unwrap();
    let ptr = (physical_offset + usable_region.start) as *mut u8;
    writeln!(Writer, "Physical memory offset: {:X}; usable range: {:p}", physical_offset, ptr).unwrap();

    let vault = unsafe { slice::from_raw_parts_mut(ptr, 100) };
    vault[0] = 65;
    vault[1] = 66;
    writeln!(Writer, "{} {}", vault[0] as char, vault[1] as char).unwrap();

    //read CR3 for current page table
    let cr3 = Cr3::read().0.start_address().as_u64();
    writeln!(Writer, "CR3 read: {:#x}", cr3).unwrap();
    // let mut cr3: u64;
    // unsafe { asm!("mov {x}, cr3", x = out(reg) cr3); }
    // writeln!(serial(), "CR3 value {:#x}", cr3).unwrap();

    let page = x86_64::structures::paging::Page::<Size4KiB>::from_start_address(VirtAddr::new(cr3)).unwrap();
    writeln!(Writer, "Page table {:?} {}", page, page.size()).unwrap();

    let page = unsafe { slice::from_raw_parts_mut((cr3 + physical_offset) as *mut usize, 6) };
    for x in 0..page.len() {
        write!(Writer, "{:?} ", page[x] as *const usize).unwrap();
    }

    writeln!(Writer, "\nEntering kernel wait loop...").unwrap();
    HandlerTable::new()
        .keyboard(key)
        .timer(tick)
        .startup(start)
        .start();
}

fn start() {
    writeln!(Writer, "Hello, world!").unwrap();
}

fn tick() {
    write!(Writer, ".").unwrap();
}

fn key(key: DecodedKey) {
    match key {
        DecodedKey::Unicode(character) => write!(Writer, "{}", character).unwrap(),
        DecodedKey::RawKey(key) => write!(Writer, "{:?}", key).unwrap(),
    }
}