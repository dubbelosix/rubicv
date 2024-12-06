use std::env;
use goblin::Object;
use std::fs;
use rubicv_emulator::instructions::PredecodedProgram;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <elf-file> <output-file>", args[0]);
        std::process::exit(1);
    }

    let buffer = fs::read(&args[1])?;

    if let Object::Elf(elf) = Object::parse(&buffer)? {
        let entry = elf.entry as u32;

        // Extract .text section
        let text_data = if let Some(text) = elf.section_headers.iter()
            .find(|&s| elf.shdr_strtab.get_at(s.sh_name) == Some(".text")) {
            &buffer[text.sh_offset as usize..(text.sh_offset + text.sh_size) as usize]
        } else {
            &[]
        };

        let text_vec = text_data.to_vec();
        let mut rubicv_elf_bytes = vec![];
        rubicv_elf_bytes.extend_from_slice(entry.to_le_bytes().as_ref());
        rubicv_elf_bytes.extend_from_slice(&text_vec);
        let _predecoded_program = PredecodedProgram::new(&rubicv_elf_bytes).unwrap();
        fs::write(&args[2], &rubicv_elf_bytes)?;
        println!("{:?}", rubicv_elf_bytes);
        println!("Raw bytes saved to {}", args[2]);
    }

    Ok(())
}
