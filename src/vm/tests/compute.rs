use crate::instructions::predecode;
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

fn encode_branch(rs1: u32, rs2: u32, func3: u32, imm: i32) -> u32 {
    let opcode = 0x63;  // Branch opcode
    let imm = (imm as u32) & 0xFFF; // 12-bit immediate
    let imm115 = (imm >> 5) & 0x7F;
    let imm40 = imm & 0x1F;
    (imm115 << 25) | (rs2 << 20) | (rs1 << 15) | (func3 << 12) | (imm40 << 7) | opcode
}

#[inline(always)]
unsafe fn zero_memory(mem: &mut [u8]) {
    mem.fill(0);
}


fn setup_compute_vm<'a>(pre_decoded_insn: &'a[PreDecodedInstruction], registers: &'a[u32; 32]) -> VM<'a> {
    let mut memory = setup_memory();

    let mut vm = VM::new(
        memory.ro_slab.as_mut() as *mut [u8],
        &mut memory.rw_slab as *mut [u8],
        pre_decoded_insn
    );
    vm.registers.copy_from_slice(registers);
    vm

}

#[test]
fn test_add() {
    // ADD x3, x1, x2
    let instruction = encode_r_type(1, 2, 3, 0x0, 0x00);
    let mut registers = [0u32; 32];
    registers[1] = 5;
    registers[2] = 7;

    let pinsn = predecode(&instruction.to_le_bytes(), CODE_START);
    let mut vm = setup_compute_vm(&pinsn, &registers);
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

    let pinsn = predecode(&instruction.to_le_bytes(), CODE_START);
    let mut vm = setup_compute_vm(&pinsn, &registers);
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

    let pinsn = predecode(&instruction.to_le_bytes(), CODE_START);
    let mut vm = setup_compute_vm(&pinsn, &registers);
    vm.step().unwrap();

    assert_eq!(vm.registers[2], 52);  // 10 + 42 = 52
    assert_eq!(vm.ppc, 1);
}

// #[test]
// fn test_x0_remains_zero() {
//     unsafe {
//         // Zero memory first
//         TEST_MEMORY.fill(0);
//         RO_SLAB.fill(0);
//         BSS_MEMORY.fill(0);
//         RW_SLAB.fill(0);
//         RO_ARGS.fill(0);
//
//         // Setup both instructions
//         let add_instruction = encode_r_type(1, 2, 0, 0x0, 0x00);
//         let bne_instruction = encode_branch(0, 0, 0x1, 8);
//
//         // Write both instructions to memory
//         TEST_MEMORY[0..4].copy_from_slice(&add_instruction.to_le_bytes());
//         TEST_MEMORY[4..8].copy_from_slice(&bne_instruction.to_le_bytes());
//
//         let mut registers = [0u32; 32];
//         registers[1] = 5;
//         registers[2] = 7;
//
//         // Create VM with the complete test memory
//         let mut vm = VM::new(
//             &TEST_MEMORY,
//             &RO_SLAB,
//             &mut BSS_MEMORY as *mut [u8],
//             &mut RW_SLAB as *mut [u8],
//             &RO_ARGS,
//             TEST_MEM, TEST_MEM, TEST_MEM, TEST_MEM, TEST_MEM,TEST_MEM
//         );
//         vm.registers.copy_from_slice(&registers);
//
//         // Execute both instructions
//         vm.step().unwrap();
//         vm.step().unwrap();
//
//         assert_eq!(vm.pc, RO_CODE_START + 8);
//         assert_eq!(vm.registers[0], 0);  // Extra check that x0 is still 0
//     }
// }

#[test]
fn test_signed_operations() {
    // Testing SLT x3, x1, x2
    let instruction = encode_r_type(1, 2, 3, 0x2, 0x00);
    let mut registers = [0u32; 32];
    registers[1] = 0xFFFFFFFF;  // -1 in two's complement
    registers[2] = 0;

    let pinsn = predecode(&instruction.to_le_bytes(), CODE_START);
    let mut vm = setup_compute_vm(&pinsn, &registers);
    vm.step().unwrap();

    assert_eq!(vm.registers[3], 1);  // -1 < 0, so result is 1
}
