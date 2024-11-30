use std::env;
use goblin::Object;
use std::fs;
use rubicv_emulator::instructions::PredecodedProgram;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <elf-file>", args[0]);
        std::process::exit(1);
    }

    let buffer = fs::read(&args[1])?;

    if let Object::Elf(elf) = Object::parse(&buffer)? {
        let entry = elf.entry;

        // Extract .text section
        let text_data = if let Some(text) = elf.section_headers.iter()
            .find(|&s| elf.shdr_strtab.get_at(s.sh_name) == Some(".text")) {
            &buffer[text.sh_offset as usize..(text.sh_offset + text.sh_size) as usize]
        } else {
            &[]
        };

        // Extract .data section
        let data_section = if let Some(data) = elf.section_headers.iter()
            .find(|&s| elf.shdr_strtab.get_at(s.sh_name) == Some(".data")) {
            &buffer[data.sh_offset as usize..(data.sh_offset + data.sh_size) as usize]
        } else {
            &[]
        };

        let text_vec = text_data.to_vec();
        let data_vec = data_section.to_vec();

        println!("Entry: {:#x}", entry);
        println!("Text size: {} bytes", text_vec.len());
        println!("Data size: {} bytes", data_vec.len());

    }

    Ok(())
}