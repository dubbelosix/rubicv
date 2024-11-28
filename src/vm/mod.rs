#[cfg(test)]
mod tests;

use core::marker::PhantomData;
#[cfg(target_os = "zkvm")]
use core::arch::asm;
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

pub trait ZeroEnforcement {
    #[inline(always)]
    fn enforce_zero(_registers: &mut [u32]) {
    }
}
pub struct EnforceZero;
pub struct NoEnforceZero;

impl ZeroEnforcement for EnforceZero {
    #[inline(always)]
    fn enforce_zero(registers: &mut [u32]) {
        unsafe { *registers.get_unchecked_mut(0) = 0 };
    }
}

impl ZeroEnforcement for NoEnforceZero {}

pub enum VMType<'a> {
    Enforced(VM<'a, EnforceZero>),
    NotEnforced(VM<'a, NoEnforceZero>),
}

pub trait VMOperations {
    fn step(&mut self) -> Result<(), RubicVError>;
    fn set_registers(&mut self, registers: &[u32]);
    fn read_u32(&self, addr: u32) -> u32;
    fn run(&mut self, arg_count: u32, max_cycles: Option<u32>) -> ExecutionResult;
    fn get_register(&self, r: u8) -> u32;
    fn get_ppc(&self) -> usize;
}

impl<'a, T: ZeroEnforcement> VMOperations for VM<'a, T> {
    fn step(&mut self) -> Result<(), RubicVError> {
        self.step()
    }

    fn set_registers(&mut self, registers: &[u32]) {
        self.registers.copy_from_slice(registers);
    }

    fn read_u32(&self, addr: u32) -> u32 {
        self.read_u32(addr)
    }

    fn run(&mut self, arg_count: u32, max_cycles: Option<u32>) -> ExecutionResult {
        self.run(arg_count, max_cycles)
    }
    fn get_register(&self, r: u8) -> u32 {
        self.registers[r as usize]
    }
    fn get_ppc(&self) -> usize {
        self.ppc
    }
}

impl<'a> VMType<'a> {
    pub fn as_operations(&mut self) -> &mut dyn VMOperations {
        match self {
            Self::Enforced(vm) => vm,
            Self::NotEnforced(vm) => vm,
        }
    }
}

impl<'a> VMType<'a> {
    pub fn new(writes_to_x0: bool,
           ro_slab: *mut [u8],
           rw_slab: *mut [u8],
           instructions: &'a [PreDecodedInstruction]) -> Self {
        if writes_to_x0 {
            Self::Enforced(VM::<EnforceZero>::new(ro_slab, rw_slab, instructions))
        } else {
            Self::NotEnforced(VM::<NoEnforceZero>::new(ro_slab, rw_slab, instructions))
        }
    }
}

pub struct VM<'a, T: ZeroEnforcement> {
    pub registers: [u32; 32],
    pub cycle_count: usize,

    pub rw_slab: *mut [u8],
    // writes are prevented anyway, but make this a const ptr sometime
    pub ro_slab: *mut [u8],

    ppc: usize, // pre-decoded program counter
    pre_decoded_instructions: &'a [PreDecodedInstruction], // pre-decoded store
    _phantom: PhantomData<T>,
}

impl<'a, T: ZeroEnforcement> VM<'a, T> {
    pub fn new(ro_slab: *mut [u8],
               rw_slab: *mut [u8],
               pre_decoded_instructions: &[PreDecodedInstruction],
    ) -> VM<T> {
            VM::<T> {
                registers: [0; 32],
                cycle_count: 0,
                rw_slab,
                ro_slab,
                ppc: 0,
                pre_decoded_instructions,
                _phantom: PhantomData,
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
    pub fn step(&mut self) -> Result<(), RubicVError> {
        let pre_decoded_insn = unsafe { self.pre_decoded_instructions.get_unchecked(self.ppc) };

        T::enforce_zero(&mut self.registers);

        let rs1 = unsafe { *self.registers.get_unchecked(pre_decoded_insn.rs1 as usize) };
        let rs2 = unsafe { *self.registers.get_unchecked(pre_decoded_insn.rs2 as usize) };
        let rd = pre_decoded_insn.rd;
        let imm = pre_decoded_insn.imm;

        let mut next_ppc = self.ppc + 1;

        match pre_decoded_insn.kind {
            // Compute instructions
            InsnKind::ADD => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1.wrapping_add(rs2) },
            InsnKind::SUB => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1.wrapping_sub(rs2) },
            InsnKind::XOR => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1 ^ rs2 },
            InsnKind::OR => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1 | rs2 },
            InsnKind::AND => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1 & rs2 },
            InsnKind::SLL => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1 << (rs2 & 0x1f) },
            InsnKind::SRL => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1 >> (rs2 & 0x1f) },
            InsnKind::SRA => unsafe { *self.registers.get_unchecked_mut(rd as usize) = ((rs1 as i32) >> (rs2 & 0x1f)) as u32 },
            InsnKind::SLT => unsafe { *self.registers.get_unchecked_mut(rd as usize) = if (rs1 as i32) < (rs2 as i32) { 1 } else { 0 } },
            InsnKind::SLTU => unsafe { *self.registers.get_unchecked_mut(rd as usize) = if rs1 < rs2 { 1 } else { 0 } },
            InsnKind::ADDI => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1.wrapping_add(imm as u32) },
            InsnKind::XORI => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1 ^ (imm as u32) },
            InsnKind::ORI => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1 | (imm as u32) },
            InsnKind::ANDI => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1 & (imm as u32) },
            InsnKind::SLLI => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1 << (imm as u32 & 0x1f) },
            InsnKind::SRLI => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1 >> (imm as u32 & 0x1f) },
            InsnKind::SRAI => unsafe { *self.registers.get_unchecked_mut(rd as usize) = ((rs1 as i32) >> (imm as u32 & 0x1f)) as u32 },
            InsnKind::SLTI => unsafe { *self.registers.get_unchecked_mut(rd as usize) = if (rs1 as i32) < imm { 1 } else { 0 } },
            InsnKind::SLTIU => unsafe { *self.registers.get_unchecked_mut(rd as usize) = if rs1 < (imm as u32) { 1 } else { 0 } },

            // Branch instructions (no register writes)
            InsnKind::BEQ => if rs1 == rs2 { next_ppc = imm as usize },
            InsnKind::BNE => if rs1 != rs2 { next_ppc = imm as usize },
            InsnKind::BLT => if (rs1 as i32) < (rs2 as i32) { next_ppc = imm as usize },
            InsnKind::BGE => if (rs1 as i32) >= (rs2 as i32) { next_ppc = imm as usize },
            InsnKind::BLTU => if rs1 < rs2 { next_ppc = imm as usize },
            InsnKind::BGEU => if rs1 >= rs2 { next_ppc = imm as usize },

            // Jump instructions
            InsnKind::JAL => {
                unsafe {
                    if rd != 0 {
                        *self.registers.get_unchecked_mut(rd as usize) = ((self.ppc + 1) * 4) as u32;
                    }
                };
                next_ppc = imm as usize;
            },
            InsnKind::JALR => {
                unsafe { *self.registers.get_unchecked_mut(rd as usize) = ((self.ppc + 1) * 4) as u32 };
                let target_addr = rs1.wrapping_add(imm as u32) & !1;
                next_ppc = (target_addr / 4) as usize;
            },

            // Load instructions
            InsnKind::LB => {
                let addr = rs1.wrapping_add(imm as u32);
                unsafe { *self.registers.get_unchecked_mut(rd as usize) = sign_extend(self.read_u8(addr) as u32, 8) };
            },
            InsnKind::LH => {
                let addr = rs1.wrapping_add(imm as u32);
                unsafe { *self.registers.get_unchecked_mut(rd as usize) = sign_extend(self.read_u16(addr) as u32, 16) };
            },
            InsnKind::LW => {
                let addr = rs1.wrapping_add(imm as u32);
                unsafe { *self.registers.get_unchecked_mut(rd as usize) = self.read_u32(addr) };
            },
            InsnKind::LBU => {
                let addr = rs1.wrapping_add(imm as u32);
                unsafe { *self.registers.get_unchecked_mut(rd as usize) = self.read_u8(addr) as u32 };
            },
            InsnKind::LHU => {
                let addr = rs1.wrapping_add(imm as u32);
                unsafe { *self.registers.get_unchecked_mut(rd as usize) = self.read_u16(addr) as u32 };
            },

            // Store instructions (no register writes)
            InsnKind::SB => {
                let addr = rs1.wrapping_add(imm as u32);
                self.write_u8(addr, rs2 as u8);
            },
            InsnKind::SH => {
                let addr = rs1.wrapping_add(imm as u32);
                self.write_u16(addr, rs2 as u16);
            },
            InsnKind::SW => {
                let addr = rs1.wrapping_add(imm as u32);
                self.write_u32(addr, rs2);
            },

            // AUIPC instruction
            InsnKind::AUIPC => {
                unsafe {
                    *self.registers.get_unchecked_mut(rd as usize) = ((self.ppc as u32) * 4).wrapping_add(imm as u32);
                };
            },

            // LUI instruction
            InsnKind::LUI => unsafe {
                *self.registers.get_unchecked_mut(rd as usize) = imm as u32;
            },

            // System instructions
            InsnKind::ECALL => return Err(RubicVError::SystemCall(self.registers[11])),
            InsnKind::EBREAK => return Err(RubicVError::Breakpoint),

            // M extension
            InsnKind::MUL => unsafe { *self.registers.get_unchecked_mut(rd as usize) = rs1.wrapping_mul(rs2) },
            InsnKind::MULH => unsafe { *self.registers.get_unchecked_mut(rd as usize) = ((rs1 as i64).wrapping_mul(rs2 as i64) >> 32) as u32 },
            InsnKind::MULHSU => unsafe { *self.registers.get_unchecked_mut(rd as usize) = ((rs1 as i64).wrapping_mul(rs2 as u64 as i64) >> 32) as u32 },
            InsnKind::MULHU => unsafe { *self.registers.get_unchecked_mut(rd as usize) = ((rs1 as u64).wrapping_mul(rs2 as u64) >> 32) as u32 },
            InsnKind::DIV => unsafe { *self.registers.get_unchecked_mut(rd as usize) =
                if rs2 == 0 {
                    u32::MAX
                } else if rs1 as i32 == i32::MIN && rs2 as i32 == -1 {
                    rs1
                } else {
                    ((rs1 as i32).wrapping_div(rs2 as i32)) as u32
                }
            },
            InsnKind::DIVU => unsafe { *self.registers.get_unchecked_mut(rd as usize) =
                if rs2 == 0 { u32::MAX } else { rs1.wrapping_div(rs2) }
            },
            InsnKind::REM => unsafe { *self.registers.get_unchecked_mut(rd as usize) =
                if rs2 == 0 {
                    rs1
                } else if rs1 as i32 == i32::MIN && rs2 as i32 == -1 {
                    0
                } else {
                    ((rs1 as i32).wrapping_rem(rs2 as i32)) as u32
                }
            },
            InsnKind::REMU => unsafe { *self.registers.get_unchecked_mut(rd as usize) =
                if rs2 == 0 { rs1 } else { rs1.wrapping_rem(rs2) }
            },

            // Catch-all for unhandled instructions
            _ => return Err(RubicVError::IllegalInstruction),
        }

        // Update the program counter
        self.ppc = next_ppc;

        Ok(())
    }

    #[cfg(target_os = "zkvm")]
    pub fn run(&mut self, arg_count: u32, max_cycles: Option<u32>) -> ExecutionResult {
        unsafe {
            *self.registers.get_unchecked_mut(10) = arg_count;
            *self.registers.get_unchecked_mut(2) = STACK_START;

            let max = max_cycles.unwrap_or(u32::MAX);
            let mut cycle_count: u32 = 0;

            loop {
                asm!(
                "addi {0}, {0}, 1",
                inout(reg) cycle_count,
                );

                if cycle_count >= max {
                    self.cycle_count = cycle_count as usize;
                    return ExecutionResult::CycleLimitExceeded;
                }

                match self.step() {
                    Ok(()) => continue,
                    Err(RubicVError::SystemCall(val)) => {
                        self.cycle_count = cycle_count as usize;
                        return ExecutionResult::Success(val)
                    },
                    Err(RubicVError::Breakpoint) => {
                        self.cycle_count = cycle_count as usize;
                        return ExecutionResult::Breakpoint
                    },
                    Err(e) => {
                        self.cycle_count = cycle_count as usize;
                        return ExecutionResult::Error(e)
                    },
                }
            }
        }
    }
    #[cfg(not(target_os = "zkvm"))]
    pub fn run(&mut self, arg_count: u32, max_cycles: Option<u32>) -> ExecutionResult {
        unsafe {
            *self.registers.get_unchecked_mut(10) = arg_count;
            *self.registers.get_unchecked_mut(2) = STACK_START;
        }
        let max = max_cycles.unwrap_or(u32::MAX);

        loop {
            self.cycle_count += 1;
            if self.cycle_count >= max as usize {
                return ExecutionResult::CycleLimitExceeded;
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
#[inline(always)]
fn sign_extend(value: u32, bits: u32) -> u32 {
    let shift = 32 - bits;
    ((value << shift) as i32 >> shift) as u32
}

