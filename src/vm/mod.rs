#[cfg(test)]
mod tests;

use crate::instructions::{DecodedInstruction, FastDecodeTable, InsnCategory, InsnKind};
use crate::errors::RubicVError;
use crate::memory::*;

#[derive(Debug)]
pub enum ExecutionResult {
    Success(u32),
    Breakpoint,
    CycleLimitExceeded,
    Error(RubicVError),
}
pub struct VM {
    pub registers: [u32; 32],
    pc: u32,
    pub cycle_count: usize,

    pub rw_slab: *mut [u8],
    pub ro_slab: *mut [u8],
    decoder: FastDecodeTable,
}



impl VM {
    pub fn new(ro_slab: *mut [u8],
               rw_slab: *mut [u8],
    ) -> VM {

        VM {
            registers: [0;32],
            pc: CODE_START,
            cycle_count: 0,
            rw_slab,
            ro_slab,
            decoder: FastDecodeTable::default()
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

    #[inline(always)]
    pub fn fetch_instruction(&self) -> u32 {
        unsafe {
            // We know PC is in RW region (specifically CODE section) and 4-byte aligned
            debug_assert!(self.pc >= CODE_START && self.pc < CODE_START + CODE_SIZE);
            let ptr = (self.rw_slab as *const u32).add((self.pc & RW_MASK) as usize >> 2);
            *ptr
        }
    }

    pub fn step(&mut self) -> Result<(), RubicVError> {
        // println!("PC: 0x{:08x}", self.pc);

        // let word = self.read_u32(self.pc);
        let word = self.fetch_instruction();

        // println!("Instruction word: 0x{:08x}", word);
        let decoded = DecodedInstruction::new(word);
        // println!("{:?}", decoded);
        let insn = self.decoder.lookup(&decoded);

        // Always ensure x0 is 0
        self.registers[0] = 0;

        match insn.category {
            InsnCategory::Compute => self.step_compute(insn.kind, &decoded)?,
            InsnCategory::System => self.step_system(insn.kind, &decoded)?,
            InsnCategory::Load => self.step_load(insn.kind, &decoded)?,
            InsnCategory::Store => self.step_store(insn.kind, &decoded)?,
            InsnCategory::Invalid => return Err(RubicVError::IllegalInstruction),
        }

        self.cycle_count += insn.cycles;
        Ok(())
    }

    fn step_load(&mut self, kind: InsnKind, decoded: &DecodedInstruction) -> Result<(), RubicVError> {
        let rs1 = self.registers[decoded.rs1 as usize];
        let addr = rs1.wrapping_add(decoded.imm_i());

        let out = match kind {
            InsnKind::LB => {
                let byte = self.read_u8(addr);
                sign_extend(byte as u32, 8)
            }
            InsnKind::LH => {
                let half = self.read_u16(addr);
                sign_extend(half as u32, 16)
            }
            InsnKind::LW => {
                self.read_u32(addr)
            }
            InsnKind::LBU => {
                self.read_u8(addr) as u32
            }
            InsnKind::LHU => {
                self.read_u16(addr) as u32
            }
            _ => return Err(RubicVError::IllegalInstruction),
        };

        self.registers[decoded.rd as usize] = out;

        self.pc += 4;
        Ok(())
    }

    fn step_store(&mut self, kind: InsnKind, decoded: &DecodedInstruction) -> Result<(), RubicVError> {
        let rs1 = self.registers[decoded.rs1 as usize];
        let rs2 = self.registers[decoded.rs2 as usize];
        let addr = rs1.wrapping_add(decoded.imm_s());

        match kind {
            InsnKind::SB => {
                self.write_u8(addr, rs2 as u8);
            }
            InsnKind::SH => {
                self.write_u16(addr, rs2 as u16);
            }
            InsnKind::SW => {
                self.write_u32(addr, rs2);
            }
            _ => return Err(RubicVError::IllegalInstruction),
        }

        self.pc += 4;
        Ok(())
    }

    fn step_compute(&mut self, kind: InsnKind, decoded: &DecodedInstruction)
                    -> Result<(), RubicVError> {
        let mut new_pc = self.pc + 4;
        let mut rd = decoded.rd;
        let rs1 = self.registers[decoded.rs1 as usize];
        let rs2 = self.registers[decoded.rs2 as usize];
        let imm_i = decoded.imm_i();

        let mut br_cond = |cond| -> u32 {
            rd = 0;
            if cond {
                new_pc = self.pc.wrapping_add(decoded.imm_b());
            }
            0
        };

        let out = match kind {
            InsnKind::ADD => rs1.wrapping_add(rs2),
            InsnKind::SUB => rs1.wrapping_sub(rs2),
            InsnKind::XOR => rs1 ^ rs2,
            InsnKind::OR => rs1 | rs2,
            InsnKind::AND => rs1 & rs2,
            InsnKind::SLL => rs1 << (rs2 & 0x1f),
            InsnKind::SRL => rs1 >> (rs2 & 0x1f),
            InsnKind::SRA => ((rs1 as i32) >> (rs2 & 0x1f)) as u32,
            InsnKind::SLT => {
                if (rs1 as i32) < (rs2 as i32) { 1 } else { 0 }
            }
            InsnKind::SLTU => {
                if rs1 < rs2 { 1 } else { 0 }
            }
            InsnKind::ADDI => rs1.wrapping_add(imm_i),
            InsnKind::XORI => rs1 ^ imm_i,
            InsnKind::ORI => rs1 | imm_i,
            InsnKind::ANDI => rs1 & imm_i,
            InsnKind::SLLI => rs1 << (imm_i & 0x1f),
            InsnKind::SRLI => rs1 >> (imm_i & 0x1f),
            InsnKind::SRAI => ((rs1 as i32) >> (imm_i & 0x1f)) as u32,
            InsnKind::SLTI => {
                if (rs1 as i32) < (imm_i as i32) { 1 } else { 0 }
            }
            InsnKind::SLTIU => {
                if rs1 < imm_i { 1 } else { 0 }
            }
            InsnKind::BEQ => br_cond(rs1 == rs2),
            InsnKind::BNE => br_cond(rs1 != rs2),
            InsnKind::BLT => br_cond((rs1 as i32) < (rs2 as i32)),
            InsnKind::BGE => br_cond((rs1 as i32) >= (rs2 as i32)),
            InsnKind::BLTU => br_cond(rs1 < rs2),
            InsnKind::BGEU => br_cond(rs1 >= rs2),
            InsnKind::JAL => {
                new_pc = self.pc.wrapping_add(decoded.imm_j());
                self.pc + 4
            }
            InsnKind::JALR => {
                let next_pc = self.pc + 4;
                new_pc = rs1.wrapping_add(imm_i) & !1;
                next_pc
            }
            InsnKind::LUI => decoded.imm_u(),
            InsnKind::AUIPC => self.pc.wrapping_add(decoded.imm_u()),
            InsnKind::MUL => rs1.wrapping_mul(rs2),
            InsnKind::MULH => {
                ((rs1 as i64).wrapping_mul(rs2 as i64) >> 32) as u32
            }
            InsnKind::MULHSU => {
                ((rs1 as i64).wrapping_mul(rs2 as u64 as i64) >> 32) as u32
            }
            InsnKind::MULHU => {
                ((rs1 as u64).wrapping_mul(rs2 as u64) >> 32) as u32
            }
            InsnKind::DIV => {
                if rs2 == 0 {
                    u32::MAX
                } else if rs1 as i32 == i32::MIN && rs2 as i32 == -1 {
                    rs1
                } else {
                    ((rs1 as i32).wrapping_div(rs2 as i32)) as u32
                }
            }
            InsnKind::DIVU => {
                if rs2 == 0 {
                    u32::MAX
                } else {
                    rs1.wrapping_div(rs2)
                }
            }
            InsnKind::REM => {
                if rs2 == 0 {
                    rs1
                } else if rs1 as i32 == i32::MIN && rs2 as i32 == -1 {
                    0
                } else {
                    ((rs1 as i32).wrapping_rem(rs2 as i32)) as u32
                }
            }
            InsnKind::REMU => {
                if rs2 == 0 {
                    rs1
                } else {
                    rs1.wrapping_rem(rs2)
                }
            }
            _ => return Err(RubicVError::IllegalInstruction),
        };

        if new_pc % 4 != 0 {
            return Err(RubicVError::MisalignedAccess);
        }

        self.registers[rd as usize] = out;

        self.pc = new_pc;
        Ok(())
    }

    fn step_system(&mut self, kind: InsnKind, _decoded: &DecodedInstruction)
                   -> Result<(), RubicVError> {
        match kind {
            InsnKind::ECALL => {
                // Standard RISC-V ECALL - returns with value from a0 (x10)
                // println!("ECALL a0: {} pc: {}",self.registers[10], self.pc);
                Err(RubicVError::SystemCall(self.registers[11]))
            }
            InsnKind::EBREAK => {
                // Breakpoint exception
                Err(RubicVError::Breakpoint)
            }
            InsnKind::MRET | _ => Err(RubicVError::IllegalInstruction),
        }
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

