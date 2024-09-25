// Copyright 2024 Kevin Ludwig
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, sync_unsafe_cell)]

use core::{arch, hint, slice};

use tracing::{error, info};

#[macro_use]
extern crate alloc;

mod int;
mod mm;
mod sm;
mod tty;
mod util;

#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]
mod multiboot {
    include!(concat!(env!("OUT_DIR"), "/multiboot.rs"));
}

extern "C" {
    static KERNEL_LMA: u8;
    static KERNEL_VMA: u8;
}

#[cfg(target_arch = "x86")]
arch::global_asm!(include_str!("x86.S"));
#[cfg(target_arch = "x86_64")]
arch::global_asm!(include_str!("x86_64.S"));

#[no_mangle]
fn main(_multiboot_magic: u32, multiboot_info: u32) -> ! {
    mm::sm::init();
    int::init();

    let multiboot_info = unsafe {
        &*((multiboot_info as usize + (&KERNEL_VMA as *const u8 as usize))
            as *const multiboot::multiboot_info)
    };

    mm::init_virt_mem();
    mm::init_phys_mem_bare();
    mm::init_phys_mem_e820(unsafe {
        slice::from_raw_parts(
            (multiboot_info.mmap_addr as usize + (&KERNEL_VMA as *const u8 as usize))
                as *const multiboot::multiboot_mmap_entry,
            multiboot_info.mmap_length as usize / size_of::<multiboot::multiboot_mmap_entry>(),
        )
    });

    tty::init_logging();

    info!("Meerkat Operating System {}", env!("CARGO_PKG_VERSION"));

    panic!("It is now safe to turn off your machine")
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info.message());

    loop {
        hint::spin_loop();
    }
}
