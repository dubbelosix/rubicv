/// TODO: Include Apache 2.0 license header and attribution to RISC0
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InsnKind {
    INVALID,
    ADD,
    SUB,
    XOR,
    OR,
    AND,
    SLL,
    SRL,
    SRA,
    SLT,
    SLTU,
    ADDI,
    XORI,
    ORI,
    ANDI,
    SLLI,
    SRLI,
    SRAI,
    SLTI,
    SLTIU,
    BEQ,
    BNE,
    BLT,
    BGE,
    BLTU,
    BGEU,
    JAL,
    JALR,
    LUI,
    AUIPC,
    MUL,
    MULH,
    MULHSU,
    MULHU,
    DIV,
    DIVU,
    REM,
    REMU,
    LB,
    LH,
    LW,
    LBU,
    LHU,
    SB,
    SH,
    SW,
    ECALL,
    EBREAK,
    MRET,
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
    pub(crate) category: InsnCategory,
    pub(crate)  opcode: u32,
    pub(crate)  func3: u32,
    pub(crate)  func7: u32,
    pub(crate)  cycles: usize,
}

type InstructionTable = [Instruction; 49];
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
    insn(InsnKind::MRET, InsnCategory::System, 0x73, 0x0, 0x18, 1),
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
    #[inline(always)]
    pub(crate) fn imm_b(&self) -> u32 {
        (self.top_bit * 0xfffff000)
            | ((self.rd & 1) << 11)
            | ((self.func7 & 0x3f) << 5)
            | (self.rd & 0x1e)
    }
    #[inline(always)]
    pub(crate) fn imm_i(&self) -> u32 {
        (self.top_bit * 0xfffff000) | (self.func7 << 5) | self.rs2
    }
    #[inline(always)]
    pub(crate) fn imm_s(&self) -> u32 {
        (self.top_bit * 0xfffff000) | (self.func7 << 5) | self.rd
    }
    #[inline(always)]
    pub(crate) fn imm_j(&self) -> u32 {
        (self.top_bit * 0xfff00000)
            | (self.rs1 << 15)
            | (self.func3 << 12)
            | ((self.rs2 & 1) << 11)
            | ((self.func7 & 0x3f) << 5)
            | (self.rs2 & 0x1e)
    }
    #[inline(always)]
    pub(crate) fn imm_u(&self) -> u32 {
        self.insn & 0xfffff000
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
    pub kind: u8,
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub imm: u32,
}

pub fn predecode(code: &[u8], code_start: u32) -> Vec<u8> {
    let mut predecoded_code = Vec::with_capacity((code.len() / 4) * 8);
    let decoder = FastDecodeTable::new();

    let _num_instructions = code.len() / 4;

    for (i, chunk) in code.chunks(4).enumerate() {
        if chunk.len() < 4 {
            break;
        }

        let code_addr = code_start + (i as u32) * 4;
        let predecoded_offset = (code_addr - code_start) * 2;


        let insn_word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let decoded = DecodedInstruction::new(insn_word);
        let insn = decoder.lookup(&decoded);

        let kind_u8 = insn.kind as u8;
        let rd = decoded.rd as u8;
        let rs1 = decoded.rs1 as u8;
        let rs2 = decoded.rs2 as u8;
        let mut imm = 0u32;

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
                let imm_b = decoded.imm_b() as i32;
                let target_predecoded_offset = predecoded_offset as i32 + imm_b * 2;
                imm = target_predecoded_offset as u32;
            }
            // JAL instruction
            InsnKind::JAL => {
                let imm_j = decoded.imm_j() as i32;
                let target_predecoded_offset = predecoded_offset as i32 + imm_j * 2;
                imm = target_predecoded_offset as u32;
            }
            // LUI and AUIPC instructions
            InsnKind::LUI => {
                imm = decoded.imm_u();
            }
            InsnKind::AUIPC => {
                imm = decoded.imm_u().wrapping_add(code_addr);
            }
            // Other instructions
            _ => {}
        }


        let pre_decoded_insn = PreDecodedInstruction {
            kind: kind_u8,
            rd,
            rs1,
            rs2,
            imm,
        };

        predecoded_code.push(pre_decoded_insn.kind);
        predecoded_code.push(pre_decoded_insn.rd);
        predecoded_code.push(pre_decoded_insn.rs1);
        predecoded_code.push(pre_decoded_insn.rs2);
        predecoded_code.extend_from_slice(&pre_decoded_insn.imm.to_le_bytes());
    }

    predecoded_code
}

