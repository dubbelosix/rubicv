#[cfg(test)]
mod tests;

use crate::instructions::{InsnKind, PreDecodedInstruction};
use crate::errors::RubicVError;
use crate::memory::*;

#[derive(Debug)]
pub enum ExecutionResult {
    Success(u32),
    Breakpoint,
    CycleLimitExceeded,
    Error(RubicVError),
}
pub struct VM<'a> {
    pub registers: [u32; 32],
    pub cycle_count: usize,

    pub rw_slab: *mut [u8],
    // writes are prevented anyway, but make this a const ptr sometime
    pub ro_slab: *mut [u8],

    ppc: usize, // pre-decoded program counter
    pre_decoded_instructions: &'a [PreDecodedInstruction], // pre-decoded store
}


impl VM<'_> {
    pub fn new(ro_slab: *mut [u8],
               rw_slab: *mut [u8],
               pre_decoded_instructions: &[PreDecodedInstruction],
    ) -> VM {

        VM {
            registers: [0;32],
            cycle_count: 0,
            rw_slab,
            ro_slab,
            ppc: 0,
            pre_decoded_instructions,
        }
    }

    #[inline(always)]
    pub fn read_u32(&self, addr: u32) -> u32 {
        unsafe {
            let ptr = if addr < RO_START {
                (self.rw_slab as *const u32).add((addr & RW_MASK) as usize >> 2)
            } else {
                (self.ro_slab as *const u32).add((addr & RO_MASK) as usize >> 2)
            };
            *ptr
        }
    }

    #[inline(always)]
    pub fn read_u16(&self, addr: u32) -> u16 {
        unsafe {
            let ptr = if addr < RO_START {
                (self.rw_slab as *const u16).add((addr & RW_MASK) as usize >> 1)
            } else {
                (self.ro_slab as *const u16).add((addr & RO_MASK) as usize >> 1)
            };
            *ptr
        }
    }

    #[inline(always)]
    pub fn read_u8(&self, addr: u32) -> u8 {
        unsafe {
            let ptr = if addr < RO_START {
                (self.rw_slab as *const u8).add((addr & RW_MASK) as usize)
            } else {
                (self.ro_slab as *const u8).add((addr & RO_MASK) as usize)
            };
            *ptr
        }
    }

    #[inline(always)]
    pub fn write_u32(&mut self, addr: u32, value: u32) {
        unsafe {
            let ptr = (self.rw_slab as *mut u32).add((addr & RW_MASK) as usize >> 2);
            *ptr = value;
        }
    }

    #[inline(always)]
    pub fn write_u16(&mut self, addr: u32, value: u16) {
        unsafe {
            let ptr = (self.rw_slab as *mut u16).add((addr & RW_MASK) as usize >> 1);
            *ptr = value;
        }
    }

    #[inline(always)]
    pub fn write_u8(&mut self, addr: u32, value: u8) {
        unsafe {
            let ptr = (self.rw_slab as *mut u8).add((addr & RW_MASK) as usize);
            *ptr = value;
        }
    }

    #[inline(always)]
    pub fn read_i8(&self, addr: u32) -> i8 {
        self.read_u8(addr) as i8
    }

    #[inline(always)]
    pub fn read_i16(&self, addr: u32) -> i16 {
        self.read_u16(addr) as i16
    }

    pub fn step(&mut self) -> Result<(), RubicVError> {
        let pre_decoded_insn = &self.pre_decoded_instructions[self.ppc];
        self.registers[0] = 0; // Always ensure x0 is 0
        let rs1 = self.registers[pre_decoded_insn.rs1 as usize];
        let rs2 = self.registers[pre_decoded_insn.rs2 as usize];
        let rd = pre_decoded_insn.rd;
        let imm = pre_decoded_insn.imm;

        let mut next_ppc = self.ppc + 1;
        let mut out = 0;

        match unsafe { std::mem::transmute::<u8, InsnKind>(pre_decoded_insn.kind) } {
            // Compute instructions
            InsnKind::ADD => out = rs1.wrapping_add(rs2),
            InsnKind::SUB => out = rs1.wrapping_sub(rs2),
            InsnKind::XOR => out = rs1 ^ rs2,
            InsnKind::OR => out = rs1 | rs2,
            InsnKind::AND => out = rs1 & rs2,
            InsnKind::SLL => out = rs1 << (rs2 & 0x1f),
            InsnKind::SRL => out = rs1 >> (rs2 & 0x1f),
            InsnKind::SRA => out = ((rs1 as i32) >> (rs2 & 0x1f)) as u32,
            InsnKind::SLT => out = if (rs1 as i32) < (rs2 as i32) { 1 } else { 0 },
            InsnKind::SLTU => out = if rs1 < rs2 { 1 } else { 0 },
            InsnKind::ADDI => out = rs1.wrapping_add(imm),
            InsnKind::XORI => out = rs1 ^ imm,
            InsnKind::ORI => out = rs1 | imm,
            InsnKind::ANDI => out = rs1 & imm,
            InsnKind::SLLI => out = rs1 << (imm & 0x1f),
            InsnKind::SRLI => out = rs1 >> (imm & 0x1f),
            InsnKind::SRAI => out = ((rs1 as i32) >> (imm & 0x1f)) as u32,
            InsnKind::SLTI => out = if (rs1 as i32) < (imm as i32) { 1 } else { 0 },
            InsnKind::SLTIU => out = if rs1 < imm { 1 } else { 0 },

            // Branch instructions
            InsnKind::BEQ => if rs1 == rs2 { next_ppc = (imm / 8) as usize },
            InsnKind::BNE => if rs1 != rs2 { next_ppc = (imm / 8) as usize },
            InsnKind::BLT => if (rs1 as i32) < (rs2 as i32) { next_ppc = (imm / 8) as usize },
            InsnKind::BGE => if (rs1 as i32) >= (rs2 as i32) { next_ppc = (imm / 8) as usize },
            InsnKind::BLTU => if rs1 < rs2 { next_ppc = (imm / 8) as usize },
            InsnKind::BGEU => if rs1 >= rs2 { next_ppc = (imm / 8) as usize },

            // Jump instructions
            InsnKind::JAL => {
                out = (self.ppc as u32 + 1) * 8; // Return address
                next_ppc = (imm / 8) as usize;
            },
            InsnKind::JALR => {
                out = (self.ppc as u32 + 1) * 8; // Return address
                next_ppc = ((rs1.wrapping_add(imm) & !1) / 8) as usize;
            },

            // Load instructions
            InsnKind::LB => {
                let addr = rs1.wrapping_add(imm);
                out = sign_extend(self.read_u8(addr) as u32, 8);
            },
            InsnKind::LH => {
                let addr = rs1.wrapping_add(imm);
                out = sign_extend(self.read_u16(addr) as u32, 16);
            },
            InsnKind::LW => {
                let addr = rs1.wrapping_add(imm);
                out = self.read_u32(addr);
            },
            InsnKind::LBU => {
                let addr = rs1.wrapping_add(imm);
                out = self.read_u8(addr) as u32;
            },
            InsnKind::LHU => {
                let addr = rs1.wrapping_add(imm);
                out = self.read_u16(addr) as u32;
            },

            // Store instructions
            InsnKind::SB => {
                let addr = rs1.wrapping_add(imm);
                self.write_u8(addr, rs2 as u8);
            },
            InsnKind::SH => {
                let addr = rs1.wrapping_add(imm);
                self.write_u16(addr, rs2 as u16);
            },
            InsnKind::SW => {
                let addr = rs1.wrapping_add(imm);
                self.write_u32(addr, rs2);
            },

            // Other instructions
            InsnKind::LUI => out = imm,
            InsnKind::AUIPC => out = (self.ppc as u32 * 8).wrapping_add(imm),

            // System instructions
            InsnKind::ECALL => return Err(RubicVError::SystemCall(self.registers[11])),
            InsnKind::EBREAK => return Err(RubicVError::Breakpoint),

            // M extension
            InsnKind::MUL => out = rs1.wrapping_mul(rs2),
            InsnKind::MULH => out = ((rs1 as i64).wrapping_mul(rs2 as i64) >> 32) as u32,
            InsnKind::MULHSU => out = ((rs1 as i64).wrapping_mul(rs2 as u64 as i64) >> 32) as u32,
            InsnKind::MULHU => out = ((rs1 as u64).wrapping_mul(rs2 as u64) >> 32) as u32,
            InsnKind::DIV => out = if rs2 == 0 {
                u32::MAX
            } else if rs1 as i32 == i32::MIN && rs2 as i32 == -1 {
                rs1
            } else {
                ((rs1 as i32).wrapping_div(rs2 as i32)) as u32
            },
            InsnKind::DIVU => out = if rs2 == 0 { u32::MAX } else { rs1.wrapping_div(rs2) },
            InsnKind::REM => out = if rs2 == 0 {
                rs1
            } else if rs1 as i32 == i32::MIN && rs2 as i32 == -1 {
                0
            } else {
                ((rs1 as i32).wrapping_rem(rs2 as i32)) as u32
            },
            InsnKind::REMU => out = if rs2 == 0 { rs1 } else { rs1.wrapping_rem(rs2) },

            _ => return Err(RubicVError::IllegalInstruction),
        }

        if !matches!(unsafe { std::mem::transmute::<u8, InsnKind>(pre_decoded_insn.kind) },
                            InsnKind::SB | InsnKind::SH | InsnKind::SW |
                            InsnKind::BEQ | InsnKind::BNE | InsnKind::BLT |
                            InsnKind::BGE | InsnKind::BLTU | InsnKind::BGEU) {
            self.registers[rd as usize] = out;
        }

        // Update PC
        self.ppc = next_ppc;
        self.cycle_count += 1;

        Ok(())
    }

    pub fn run(&mut self, arg_count: u32, max_cycles: Option<u32>) -> ExecutionResult {
        self.registers[10] = arg_count;
        self.registers[2] = STACK_START;

        loop {
            if let Some(max) = max_cycles {
                if self.cycle_count >= max as usize {
                    return ExecutionResult::CycleLimitExceeded;
                }
            }

            match self.step() {
                Ok(()) => continue,
                Err(RubicVError::SystemCall(val)) => return ExecutionResult::Success(val),
                Err(RubicVError::Breakpoint) => return ExecutionResult::Breakpoint,
                Err(e) => return ExecutionResult::Error(e),
            }
        }
    }

}

fn sign_extend(value: u32, bits: u32) -> u32 {
    let shift = 32 - bits;
    ((value << shift) as i32 >> shift) as u32
}

