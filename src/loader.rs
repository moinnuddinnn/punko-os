use uefi::boot;
use uefi::cstr16;

use uefi::proto::media::file::{
    File,
    FileAttribute,
    FileInfo,
    FileMode,
};

use crate::elf::{
    //Elf64Header,
    Elf64ProgramHeader,
};

use uefi::boot::{
    AllocateType,
    MemoryType,
};


pub fn load_kernel(image_handle: uefi::Handle) {
    // Get the filesystem from which this bootloader was loaded.
    let mut fs = boot::get_image_file_system(image_handle)
        .expect("Failed to get filesystem");

    // Open the root directory.
    let mut root = fs
        .open_volume()
        .expect("Failed to open root directory");

    // Open kernel.elf
    let handle = match root.open(
        cstr16!("kernel.elf"),
        FileMode::Read,
        FileAttribute::empty(),
    ) {
        Ok(handle) => {
            uefi::println!("Kernel found!");
            handle
        }

        Err(err) => {
            uefi::println!("Kernel NOT found!");
            uefi::println!("{:?}", err.status());
            return;
        }
    };

    // Convert the generic file handle into a regular file.
    let mut file = handle
        .into_regular_file()
        .expect("kernel.elf is not a regular file");

    // Read file metadata.
    let mut info_buf = [0u8; 512];

    let info = file
        .get_info::<FileInfo>(&mut info_buf)
        .expect("Failed to get file information");

    let kernel_size = info.file_size() as usize;

    uefi::println!("Kernel size: {} bytes", kernel_size);

    // Allocate a buffer large enough to hold the kernel
    uefi::println!("Allocating kernel buffer...");

    let mut kernel_data = vec![0u8; kernel_size];

    uefi::println!("Reading kernel into memory...");

    // Read the entire kernel.
    let bytes_read = file
        .read(&mut kernel_data)
        .expect("Failed to read kernel");

    uefi::println!("Read {} bytes.", bytes_read);

    if &kernel_data[..4] == b"\x7FELF" {
        uefi::println!("Valid ELF executable found.");
    } else {
        uefi::println!("kernel.elf is not a valid ELF executable.");
    }

    // Parse the ELF header.

    let header = unsafe {
    &*(kernel_data.as_ptr() as *const crate::elf::Elf64Header)
    };

    uefi::println!("Entry point: {:#018x}", header.entry);
    uefi::println!("Program header offset: {}", header.phoff);
    uefi::println!("Program headers: {}", header.phnum);
    uefi::println!("Machine: {}", header.machine);
    uefi::println!("ELF type: {}", header.elf_type);
    // TODO:

    uefi::println!("----------------------------------------------");
    for i in 0..header.phnum {

        let offset = header.phoff as usize + i as usize * header.phentsize as usize;

        let program_header = unsafe {

            &*(kernel_data
                .as_ptr()
                .add(offset)
                as *const Elf64ProgramHeader)
        };

        uefi::println!("Program Header {}:", i);

        if program_header.p_type != 1 {
            uefi::println!("Not Loadable. \n");
            continue;
        }

        uefi::println!("PT_LOAD segment found.");

        let pages = (program_header.p_memsz as usize + 4095) / 4096;

        uefi::println!("Allocating {} pages for segment...", pages);

        let memory = boot::allocate_pages(
            AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            pages,
        )
        .expect("Failed to allocate memory for segment");
        
        uefi::println!("Segment allocated at: {:p}", memory.as_ptr());

        unsafe {
            core::ptr::copy_nonoverlapping(kernel_data.as_ptr().add(program_header.p_offset as usize), memory.as_ptr(), program_header.p_filesz as usize);

        }

        uefi::println!("Segment loaded into memory.");


        if program_header.p_memsz > program_header.p_filesz {
            unsafe {
                core::ptr::write_bytes(memory.as_ptr().add(program_header.p_filesz as usize), 0, (program_header.p_memsz - program_header.p_filesz) as usize);
            }
        }

        uefi::println!("Zerped {} extra bytes.", program_header.p_memsz - program_header.p_filesz);


        uefi::println!("  Type: {}", program_header.p_type);
        uefi::println!("  Offset: {:#x}", program_header.p_offset);
        uefi::println!("  Virtual Address: {:#x}", program_header.p_vaddr);
        uefi::println!("  File Size: {}", program_header.p_filesz);
        uefi::println!("  Memory Size: {}", program_header.p_memsz);
        uefi::println!("  Flags: {:#x}", program_header.p_flags);

        uefi::println!("----------------------------------------------");
    };



    // Parse the ELF header.
    // Load ELF segments into memory.
    // Jump to the kernel entry point.
}
