use alloc::vec::Vec;
use crate::errors::RubicVError;
use crate::memory::CODE_SIZE;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum InsnKind {
    INVALID = 0,
    ADD = 1,
    SUB = 2,
    XOR = 3,
    OR = 4,
    AND = 5,
    SLL = 6,
    SRL = 7,
    SRA = 8,
    SLT = 9,
    SLTU = 10,
    ADDI = 11,
    XORI = 12,
    ORI = 13,
    ANDI = 14,
    SLLI = 15,
    SRLI = 16,
    SRAI = 17,
    SLTI = 18,
    SLTIU = 19,
    BEQ = 20,
    BNE = 21,
    BLT = 22,
    BGE = 23,
    BLTU = 24,
    BGEU = 25,
    JAL = 26,
    JALR = 27,
    LUI = 28,
    AUIPC = 29,
    MUL = 30,
    MULH = 31,
    MULHSU = 32,
    MULHU = 33,
    DIV = 34,
    DIVU = 35,
    REM = 36,
    REMU = 37,
    LB = 38,
    LH = 39,
    LW = 40,
    LBU = 41,
    LHU = 42,
    SB = 43,
    SH = 44,
    SW = 45,
    ECALL = 46,
    EBREAK = 47,
}

#[derive(Clone, Copy, Debug)]
pub enum InsnCategory {
    Compute,
    Load,
    Store,
    System,
    Invalid,
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub(crate)  kind: InsnKind,
    #[allow(dead_code)]
    category: InsnCategory,
    pub(crate)  opcode: u32,
    pub(crate)  func3: u32,
    pub(crate)  func7: u32,
    #[allow(dead_code)]
    cycles: usize,
}

type InstructionTable = [Instruction; 48];
const fn insn(
    kind: InsnKind,
    category: InsnCategory,
    opcode: u32,
    func3: i32,
    func7: i32,
    cycles: usize,
) -> Instruction {
    Instruction {
        kind,
        category,
        opcode,
        func3: func3 as u32,
        func7: func7 as u32,
        cycles,
    }
}

const RV32IM_ISA: InstructionTable = [
    insn(InsnKind::INVALID, InsnCategory::Invalid, 0x00, 0x0, 0x00, 0),
    insn(InsnKind::ADD, InsnCategory::Compute, 0x33, 0x0, 0x00, 1),
    insn(InsnKind::SUB, InsnCategory::Compute, 0x33, 0x0, 0x20, 1),
    insn(InsnKind::XOR, InsnCategory::Compute, 0x33, 0x4, 0x00, 2),
    insn(InsnKind::OR, InsnCategory::Compute, 0x33, 0x6, 0x00, 2),
    insn(InsnKind::AND, InsnCategory::Compute, 0x33, 0x7, 0x00, 2),
    insn(InsnKind::SLL, InsnCategory::Compute, 0x33, 0x1, 0x00, 1),
    insn(InsnKind::SRL, InsnCategory::Compute, 0x33, 0x5, 0x00, 2),
    insn(InsnKind::SRA, InsnCategory::Compute, 0x33, 0x5, 0x20, 2),
    insn(InsnKind::SLT, InsnCategory::Compute, 0x33, 0x2, 0x00, 1),
    insn(InsnKind::SLTU, InsnCategory::Compute, 0x33, 0x3, 0x00, 1),
    insn(InsnKind::ADDI, InsnCategory::Compute, 0x13, 0x0, -1, 1),
    insn(InsnKind::XORI, InsnCategory::Compute, 0x13, 0x4, -1, 2),
    insn(InsnKind::ORI, InsnCategory::Compute, 0x13, 0x6, -1, 2),
    insn(InsnKind::ANDI, InsnCategory::Compute, 0x13, 0x7, -1, 2),
    insn(InsnKind::SLLI, InsnCategory::Compute, 0x13, 0x1, 0x00, 1),
    insn(InsnKind::SRLI, InsnCategory::Compute, 0x13, 0x5, 0x00, 2),
    insn(InsnKind::SRAI, InsnCategory::Compute, 0x13, 0x5, 0x20, 2),
    insn(InsnKind::SLTI, InsnCategory::Compute, 0x13, 0x2, -1, 1),
    insn(InsnKind::SLTIU, InsnCategory::Compute, 0x13, 0x3, -1, 1),
    insn(InsnKind::BEQ, InsnCategory::Compute, 0x63, 0x0, -1, 1),
    insn(InsnKind::BNE, InsnCategory::Compute, 0x63, 0x1, -1, 1),
    insn(InsnKind::BLT, InsnCategory::Compute, 0x63, 0x4, -1, 1),
    insn(InsnKind::BGE, InsnCategory::Compute, 0x63, 0x5, -1, 1),
    insn(InsnKind::BLTU, InsnCategory::Compute, 0x63, 0x6, -1, 1),
    insn(InsnKind::BGEU, InsnCategory::Compute, 0x63, 0x7, -1, 1),
    insn(InsnKind::JAL, InsnCategory::Compute, 0x6f, -1, -1, 1),
    insn(InsnKind::JALR, InsnCategory::Compute, 0x67, 0x0, -1, 1),
    insn(InsnKind::LUI, InsnCategory::Compute, 0x37, -1, -1, 1),
    insn(InsnKind::AUIPC, InsnCategory::Compute, 0x17, -1, -1, 1),
    insn(InsnKind::MUL, InsnCategory::Compute, 0x33, 0x0, 0x01, 1),
    insn(InsnKind::MULH, InsnCategory::Compute, 0x33, 0x1, 0x01, 1),
    insn(InsnKind::MULHSU, InsnCategory::Compute, 0x33, 0x2, 0x01, 1),
    insn(InsnKind::MULHU, InsnCategory::Compute, 0x33, 0x3, 0x01, 1),
    insn(InsnKind::DIV, InsnCategory::Compute, 0x33, 0x4, 0x01, 2),
    insn(InsnKind::DIVU, InsnCategory::Compute, 0x33, 0x5, 0x01, 2),
    insn(InsnKind::REM, InsnCategory::Compute, 0x33, 0x6, 0x01, 2),
    insn(InsnKind::REMU, InsnCategory::Compute, 0x33, 0x7, 0x01, 2),
    insn(InsnKind::LB, InsnCategory::Load, 0x03, 0x0, -1, 1),
    insn(InsnKind::LH, InsnCategory::Load, 0x03, 0x1, -1, 1),
    insn(InsnKind::LW, InsnCategory::Load, 0x03, 0x2, -1, 1),
    insn(InsnKind::LBU, InsnCategory::Load, 0x03, 0x4, -1, 1),
    insn(InsnKind::LHU, InsnCategory::Load, 0x03, 0x5, -1, 1),
    insn(InsnKind::SB, InsnCategory::Store, 0x23, 0x0, -1, 1),
    insn(InsnKind::SH, InsnCategory::Store, 0x23, 0x1, -1, 1),
    insn(InsnKind::SW, InsnCategory::Store, 0x23, 0x2, -1, 1),
    insn(InsnKind::ECALL, InsnCategory::System, 0x73, 0x0, 0x00, 1),
    insn(InsnKind::EBREAK, InsnCategory::System, 0x73, 0x0, 0x01, 1),
];

#[derive(Clone, Debug, Default)]
pub struct DecodedInstruction {
    pub insn: u32,
    pub top_bit: u32,
    pub func7: u32,
    pub rs2: u32,
    pub rs1: u32,
    pub func3: u32,
    pub rd: u32,
    pub opcode: u32,
}

impl DecodedInstruction {
    pub fn new(insn: u32) -> Self {
        Self {
            insn,
            top_bit: (insn & 0x80000000) >> 31,
            func7: (insn & 0xfe000000) >> 25,
            rs2: (insn & 0x01f00000) >> 20,
            rs1: (insn & 0x000f8000) >> 15,
            func3: (insn & 0x00007000) >> 12,
            rd: (insn & 0x00000f80) >> 7,
            opcode: insn & 0x0000007f,
        }
    }
    // Sign-extend a value based on the number of bits
    fn sign_extend(value: u32, bits: u32) -> i32 {
        let shift = 32 - bits;
        ((value << shift) as i32) >> shift
    }

    // Extract and reconstruct the B-type immediate (for branch instructions)
    #[inline(always)]
    pub(crate) fn imm_b(&self) -> i32 {
        let imm12 = (self.insn & 0x80000000) >> 31;
        let imm10_5 = (self.insn & 0x7E000000) >> 25;
        let imm4_1 = (self.insn & 0x00000F00) >> 8;
        let imm11 = (self.insn & 0x00000080) >> 7;
        let imm = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);
        Self::sign_extend(imm, 13)
    }

    // Extract and reconstruct the J-type immediate (for JAL instruction)
    #[inline(always)]
    pub(crate) fn imm_j(&self) -> i32 {
        let imm20 = (self.insn & 0x80000000) >> 31;
        let imm19_12 = (self.insn & 0x000FF000) >> 12;
        let imm11 = (self.insn & 0x00100000) >> 20;
        let imm10_1 = (self.insn & 0x7FE00000) >> 21;
        let imm = (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1);
        Self::sign_extend(imm, 21)
    }

    // Extract and reconstruct the I-type immediate (for ALU and load instructions)
    #[inline(always)]
    pub(crate) fn imm_i(&self) -> i32 {
        let imm = (self.insn & 0xFFF00000) >> 20;
        Self::sign_extend(imm, 12)
    }

    // Extract and reconstruct the S-type immediate (for store instructions)
    #[inline(always)]
    pub(crate) fn imm_s(&self) -> i32 {
        let imm11_5 = (self.insn & 0xFE000000) >> 25;
        let imm4_0 = (self.insn & 0x00000F80) >> 7;
        let imm = (imm11_5 << 5) | imm4_0;
        Self::sign_extend(imm, 12)
    }

    // Extract the U-type immediate (for LUI and AUIPC instructions)
    #[inline(always)]
    pub(crate) fn imm_u(&self) -> u32 {
        self.insn & 0xFFFFF000
    }
}

type FastInstructionTable = [Instruction; 1 << 10];

pub struct FastDecodeTable {
    table: FastInstructionTable,
}

impl Default for FastDecodeTable {
    fn default() -> Self {
        Self::new()
    }
}

impl FastDecodeTable {
    #[inline(always)]
    fn map10(opcode: u32, func3: u32, func7: u32) -> usize {
        let op_high = opcode >> 2;
        // Map 0 -> 0, 1 -> 1, 0x20 -> 2, everything else to 3
        let func72bits = if func7 <= 1 {
            func7
        } else if func7 == 0x20 {
            2
        } else {
            3
        };
        ((op_high << 5) | (func72bits << 3) | func3) as usize
    }
    fn new() -> Self {
        // Initialize with INVALID instruction
        let invalid_insn = insn(InsnKind::INVALID, InsnCategory::Invalid, 0x00, 0x0, 0x00, 0);
        let mut table = [invalid_insn; 1 << 10];

        // Fill table with actual instructions instead of indices
        for insn in RV32IM_ISA.iter() {
            Self::add_insn(&mut table, insn);
        }
        Self { table }
    }
    #[inline(always)]
    fn add_insn(table: &mut FastInstructionTable, insn: &Instruction) {
        let op_high = insn.opcode >> 2;
        if (insn.func3 as i32) < 0 {
            for f3 in 0..8 {
                for f7b in 0..4 {
                    let idx = (op_high << 5) | (f7b << 3) | f3;
                    table[idx as usize] = *insn;
                }
            }
        } else if (insn.func7 as i32) < 0 {
            for f7b in 0..4 {
                let idx = (op_high << 5) | (f7b << 3) | insn.func3;
                table[idx as usize] = *insn;
            }
        } else {
            table[Self::map10(insn.opcode, insn.func3, insn.func7)] = *insn;
        }
    }


    #[inline(always)]
    pub fn lookup(&self, decoded: &DecodedInstruction) -> Instruction {
        // Direct table lookup, no second array access needed
        self.table[Self::map10(decoded.opcode, decoded.func3, decoded.func7)]
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct PreDecodedInstruction {
    pub kind: InsnKind,
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub imm: i32,
}
#[derive(Clone, Debug)]
pub struct PredecodedProgram {
    pub instructions: Vec<PreDecodedInstruction>,
    pub entrypoint: usize,
    pub writes_to_x0: bool,
}

impl PredecodedProgram {
    pub fn new(elf_bytes: &[u8]) -> Result<Self, RubicVError> {
        // Ensure the header is at least 8 bytes
        if elf_bytes.len() < 4 || elf_bytes.len() > CODE_SIZE as usize + 4  {
            return Err(RubicVError::ELFDecodeError)
        }

        let code = &elf_bytes[4..];
        let entrypoint = (u32::from_le_bytes(elf_bytes.get(0..4).ok_or(RubicVError::ELFDecodeError)?.try_into().map_err(|_| RubicVError::ELFDecodeError)?) / 4) as usize;


        // Pre-decode the instructions
        let mut predecoded_instructions = Vec::with_capacity(code.len() / 4);
        let decoder = FastDecodeTable::new();
        let mut writes_to_x0 = false;

        for (i, chunk) in code.chunks(4).enumerate() {
            if chunk.len() < 4 {
                break;
            }

            // Code address is 0
            let predecoded_offset = i as i32;

            let insn_word = u32::from_le_bytes(chunk.try_into().expect("Incorrect chunk length"));

            let decoded = DecodedInstruction::new(insn_word);
            let insn = decoder.lookup(&decoded);

            let rd = decoded.rd as u8;
            let rs1 = decoded.rs1 as u8;
            let rs2 = decoded.rs2 as u8;
            let mut imm = 0i32;

            // Check for writes to x0
            match insn.kind {
                InsnKind::ADDI | InsnKind::XORI | InsnKind::ORI | InsnKind::ANDI |
                InsnKind::SLTI | InsnKind::SLTIU | InsnKind::SLLI | InsnKind::SRLI |
                InsnKind::SRAI | InsnKind::LB | InsnKind::LH | InsnKind::LW |
                InsnKind::LBU | InsnKind::LHU | InsnKind::JALR |  // removed JAL
                InsnKind::LUI | InsnKind::AUIPC | InsnKind::ADD | InsnKind::SUB |
                InsnKind::SLL | InsnKind::SLT | InsnKind::SLTU | InsnKind::XOR |
                InsnKind::SRL | InsnKind::SRA | InsnKind::OR | InsnKind::AND => {
                    if rd == 0 {
                        if !(insn.kind == InsnKind::ADDI && rs1 == 0 && imm == 0) {
                            if insn_word != 0 {
                                writes_to_x0 = true;
                            }
                        }
                    }
                }
                _ => {}
            }

            match insn.kind {
                // Immediate instructions
                InsnKind::ADDI | InsnKind::XORI | InsnKind::ORI | InsnKind::ANDI
                | InsnKind::SLTI | InsnKind::SLTIU | InsnKind::SLLI | InsnKind::SRLI
                | InsnKind::SRAI | InsnKind::LB | InsnKind::LH | InsnKind::LW
                | InsnKind::LBU | InsnKind::LHU | InsnKind::JALR => {
                    imm = decoded.imm_i();
                }
                // Store instructions
                InsnKind::SB | InsnKind::SH | InsnKind::SW => {
                    imm = decoded.imm_s();
                }
                // Branch instructions
                InsnKind::BEQ | InsnKind::BNE | InsnKind::BLT | InsnKind::BGE
                | InsnKind::BLTU | InsnKind::BGEU => {
                    let imm_b = decoded.imm_b();
                    let target_index = predecoded_offset + (imm_b / 4);
                    imm = target_index;
                }
                // JAL instruction
                InsnKind::JAL => {
                    let imm_j = decoded.imm_j();
                    let target_index = predecoded_offset + (imm_j / 4);
                    imm = target_index;
                }
                // LUI and AUIPC instructions
                InsnKind::LUI => {
                    imm = decoded.imm_u() as i32;
                }
                InsnKind::AUIPC => {
                    imm = decoded.imm_u() as i32;
                }
                // Other instructions (compute, system, etc.)
                InsnKind::ADD | InsnKind::SUB | InsnKind::XOR | InsnKind::OR | InsnKind::AND
                | InsnKind::SLL | InsnKind::SRL | InsnKind::SRA | InsnKind::SLT
                | InsnKind::SLTU | InsnKind::MUL | InsnKind::MULH | InsnKind::MULHSU
                | InsnKind::MULHU | InsnKind::DIV | InsnKind::DIVU | InsnKind::REM
                | InsnKind::REMU => {
                    // No immediate needed; set imm to 0
                    imm = 0;
                }
                _ => {}
            }

            let pre_decoded_insn = PreDecodedInstruction {
                kind: insn.kind,
                rd,
                rs1,
                rs2,
                imm,
            };
            predecoded_instructions.push(pre_decoded_insn);
        }

        Ok(PredecodedProgram {
            instructions: predecoded_instructions,
            entrypoint,
            writes_to_x0,
        })
    }

}
