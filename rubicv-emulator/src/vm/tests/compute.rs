use crate::instructions::{PredecodedProgram};
use super::*;

// R-type helper
fn encode_r_type(rs1: u32, rs2: u32, rd: u32, func3: u32, func7: u32) -> u32 {
    let opcode = 0x33;  // R-type opcode
    (func7 << 25) | (rs2 << 20) | (rs1 << 15) | (func3 << 12) | (rd << 7) | opcode
}

// I-type helper
fn encode_i_type(rs1: u32, rd: u32, func3: u32, imm: i32) -> u32 {
    let opcode = 0x13;  // I-type opcode
    let imm = (imm as u32) & 0xFFF; // 12-bit immediate
    (imm << 20) | (rs1 << 15) | (func3 << 12) | (rd << 7) | opcode
}

fn setup_compute_vm<'a>(pre_decoded_program: &'a PredecodedProgram, registers: &'a[u32; 32]) -> VM<'a, EnforceZero> {
    let mut memory = setup_memory();

    let mut vm = VM::<EnforceZero>::new(
        memory.memory_slab.as_mut() as *mut [u8],
        pre_decoded_program.entrypoint,
        &pre_decoded_program.instructions
    );
    vm.registers.copy_from_slice(registers);
    vm

}

fn setup_elf_bytes(code_bytes: &[u8]) -> Vec<u8> {
    let entry_point = [0u8;4];
    let code_len = (code_bytes.len() as u32).to_le_bytes();
    let mut elf_bytes = vec![];
    elf_bytes.extend_from_slice(&code_len);
    elf_bytes.extend_from_slice(&entry_point);
    elf_bytes.extend_from_slice(&code_bytes);
    elf_bytes
}

#[test]
fn test_add() {
    // ADD x3, x1, x2
    let instruction = encode_r_type(1, 2, 3, 0x0, 0x00);
    let mut registers = [0u32; 32];
    registers[1] = 5;
    registers[2] = 7;
    let elf_bytes =  setup_elf_bytes(&instruction.to_le_bytes());
    let predecoded_program = PredecodedProgram::new(&elf_bytes).unwrap();
    let mut vm = setup_compute_vm(&predecoded_program, &registers);
    vm.step().unwrap();

    assert_eq!(vm.registers[3], 12);  // 5 + 7 = 12
    assert_eq!(vm.ppc, 1);
}

#[test]
fn test_sub() {
    // SUB x3, x1, x2
    let instruction = encode_r_type(1, 2, 3, 0x0, 0x20);
    let mut registers = [0u32; 32];
    registers[1] = 10;
    registers[2] = 3;

    let elf_bytes =  setup_elf_bytes(&instruction.to_le_bytes());
    let predecoded_program = PredecodedProgram::new(&elf_bytes).unwrap();
    let mut vm = setup_compute_vm(&predecoded_program, &registers);
    vm.step().unwrap();

    assert_eq!(vm.registers[3], 7);  // 10 - 3 = 7
    assert_eq!(vm.ppc, 1);
}

#[test]
fn test_addi() {
    // ADDI x2, x1, 42
    let instruction = encode_i_type(1, 2, 0x0, 42);
    let mut registers = [0u32; 32];
    registers[1] = 10;

    let elf_bytes =  setup_elf_bytes(&instruction.to_le_bytes());
    let predecoded_program = PredecodedProgram::new(&elf_bytes).unwrap();
    let mut vm = setup_compute_vm(&predecoded_program, &registers);
    vm.step().unwrap();

    assert_eq!(vm.registers[2], 52);  // 10 + 42 = 52
    assert_eq!(vm.ppc, 1);
}

#[test]
fn test_signed_operations() {
    // Testing SLT x3, x1, x2
    let instruction = encode_r_type(1, 2, 3, 0x2, 0x00);
    let mut registers = [0u32; 32];
    registers[1] = 0xFFFFFFFF;  // -1 in two's complement
    registers[2] = 0;

    let elf_bytes =  setup_elf_bytes(&instruction.to_le_bytes());
    let predecoded_program = PredecodedProgram::new(&elf_bytes).unwrap();
    let mut vm = setup_compute_vm(&predecoded_program, &registers);
    vm.step().unwrap();

    assert_eq!(vm.registers[3], 1);  // -1 < 0, so result is 1
}
