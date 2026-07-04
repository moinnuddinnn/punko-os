#![no_main]

use uefi::prelude::*;
use uefi::{boot, helpers};

mod loader;
mod elf;

#[entry]
fn efi_main() -> Status {
    helpers::init().unwrap();

    uefi::println!("Hello from my bootloader!");

    loader::load_kernel(boot::image_handle());

    loop {}
}