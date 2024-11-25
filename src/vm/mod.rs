#[cfg(test)]
mod tests;

use crate::instructions::{DecodedInstruction, FastDecodeTable, InsnCategory, InsnKind};
use crate::errors::RubicVError;
use crate::memory_bounds::*;

type ReadSlice<'a> = &'a [u8];
type WriteSlice = *mut [u8];
pub struct VM<'a> {
    registers: [u32; 32],
    pc: u32,
    cycle_count: usize,

    code_memory: ReadSlice<'a>,
    ro_slab: ReadSlice<'a>,
    rw_slab: WriteSlice,
    ro_args: ReadSlice<'a>,

    bss_memory_ptr: WriteSlice,

    // Precomputes
    rw_heap_end: u32,
    rw_stack_start: u32,
    rw_stack_end: u32,
    rw_slab_end: u32,
    ro_code_end: u32,
    ro_slab_end: u32,
    ro_args_end: u32,

    decoder: FastDecodeTable,
}

#[derive(Debug)]
pub enum ExecutionResult {
    Success(u32),
    Breakpoint,
    CycleLimitExceeded,
    Error(RubicVError),
}

impl VM<'_> {
    pub fn new<'a>(code_memory: &'a [u8],
                   ro_slab: &'a [u8],
                   bss_memory_ptr: *mut [u8],
                   rw_slab: *mut [u8],
                   ro_args: &'a [u8],
                   rw_heap_maxsize: usize,
                   rw_stack_maxsize: usize,
                   ro_code_maxsize: usize,
                   ro_slab_maxsize: usize,
                   rw_slab_maxsize: usize,
                   ro_args_maxsize: usize,

    ) -> VM<'a> {

        // Precompute ranges
        let rw_heap_end = RW_HEAP_START.checked_add(rw_heap_maxsize as u32)
            .expect("Heap size overflow");
        let rw_stack_start = RW_STACK_START.checked_sub(rw_stack_maxsize as u32)
            .expect("Stack size overflow");
        let rw_stack_end = RW_STACK_START;
        let rw_slab_end = RW_CUSTOM_SLAB_START.checked_add(rw_slab_maxsize as u32)
            .expect("RW slab size overflow");
        let ro_code_end = RO_CODE_START.checked_add(ro_code_maxsize as u32)
            .expect("Code size overflow");
        let ro_slab_end = RO_CUSTOM_SLAB_START.checked_add(ro_slab_maxsize as u32)
            .expect("RO slab size overflow");
        let ro_args_end = RO_CUSTOM_ARGS_START.checked_add(ro_args_maxsize as u32)
            .expect("RO slab size overflow");

        VM {
            registers: [0;32],
            pc: RO_CODE_START,
            cycle_count: 0,
            code_memory,
            bss_memory_ptr,
            rw_slab,
            ro_slab,
            ro_args,
            rw_heap_end,
            rw_stack_start,
            rw_stack_end,
            rw_slab_end,
            ro_code_end,
            ro_slab_end,
            ro_args_end,

            decoder: FastDecodeTable::default()
        }
    }

    pub fn reset(&mut self) {
        // Zero out registers
        self.registers = [0; 32];
        // Reset program counter to start of code
        self.pc = RO_CODE_START;
        // Reset cycle count
        self.cycle_count = 0;
        // Reset stack pointer
        self.registers[2] = RW_STACK_START;

        // Zero out heap and stack memory
        unsafe {
            let heap_size = (self.rw_heap_end - RW_HEAP_START) as usize;
            let stack_size = (self.rw_stack_end - self.rw_stack_start) as usize;
            if let Some(slice) = (*self.bss_memory_ptr).get_mut(..heap_size + stack_size) {
                slice.fill(0);
            }
        }

        // Zero out RW slab
        unsafe {
            if let Some(slice) = (*self.rw_slab).get_mut(..) {
                slice.fill(0);
            }
        }
    }


    #[inline(always)]
    fn get_region_type(&self, addr: u32) -> u8 {
        REGION_TABLE[(addr >> 28) as usize]
    }

    #[inline(always)]
    fn is_addr_valid_for_writes(&self, addr: u32) -> Result<(WriteSlice, usize), RubicVError> {
        match self.get_region_type(addr) {
            REGION_RW => {
                if addr >= RW_HEAP_START && addr < self.rw_heap_end {
                    Ok((self.bss_memory_ptr, (addr - RW_HEAP_START) as usize))
                } else if addr >= self.rw_stack_start && addr < self.rw_stack_end {
                    Ok((self.bss_memory_ptr, (self.rw_stack_end - addr) as usize))
                } else if addr >= RW_CUSTOM_SLAB_START && addr < self.rw_slab_end {
                    Ok((self.rw_slab, (addr - RW_CUSTOM_SLAB_START) as usize))
                } else {
                    Err(RubicVError::MemoryWriteOutOfBounds)
                }
            }
            _ => Err(RubicVError::MemoryWriteOutOfBounds),
        }
    }

    #[inline(always)]
    fn is_addr_valid_for_reads(&self, addr: u32) -> Result<(ReadSlice, usize), RubicVError> {
        match self.get_region_type(addr) {
            REGION_RW => {
                if addr >= RW_HEAP_START && addr < self.rw_heap_end {
                    unsafe {
                        Ok((&*self.bss_memory_ptr, (addr - RW_HEAP_START) as usize))
                    }
                } else if addr >= self.rw_stack_start && addr < self.rw_stack_end {
                    unsafe {
                        Ok((&*self.bss_memory_ptr, (self.rw_stack_end - addr) as usize))
                    }
                } else if addr >= RW_CUSTOM_SLAB_START && addr < self.rw_slab_end {
                    unsafe {
                        Ok((&*self.rw_slab, (addr - RW_CUSTOM_SLAB_START) as usize))
                    }
                } else {
                    Err(RubicVError::MemoryReadOutOfBounds)
                }
            }
            REGION_RO => {
                if addr >= RO_CODE_START && addr < self.ro_code_end {
                    Ok((self.code_memory, (addr - RO_CODE_START) as usize))
                } else if addr >= RO_CUSTOM_SLAB_START && addr < self.ro_slab_end {
                    Ok((self.ro_slab, (addr - RO_CUSTOM_SLAB_START) as usize))
                } else if addr >= RO_CUSTOM_ARGS_START && addr < self.ro_args_end {
                    Ok((self.ro_args, (addr - RO_CUSTOM_ARGS_START) as usize))
                } else {
                    Err(RubicVError::MemoryReadOutOfBounds)
                }
            }
            _ => Err(RubicVError::MemoryReadOutOfBounds),
        }
    }

    /// Load Byte (signed)
    #[inline(always)]
    pub fn read_i8(&self, addr: u32) -> Result<i8, RubicVError> {
        let (slice, offset) = self.is_addr_valid_for_reads(addr)?;
        slice.get(offset)
            .map(|&b| b as i8)
            .ok_or(RubicVError::MemoryReadOutOfBounds)
    }

    /// Load Byte Unsigned
    #[inline(always)]
    pub fn read_u8(&self, addr: u32) -> Result<u8, RubicVError> {
        let (slice, offset) = self.is_addr_valid_for_reads(addr)?;
        slice.get(offset)
            .copied()
            .ok_or(RubicVError::MemoryReadOutOfBounds)
    }

    /// Load Half-word (signed)
    #[inline(always)]
    pub fn read_i16(&self, addr: u32) -> Result<i16, RubicVError> {
        if addr % 2 != 0 {
            return Err(RubicVError::MemoryMisaligned);
        }
        let (slice, offset) = self.is_addr_valid_for_reads(addr)?;
        if offset + 2 > slice.len() {
            return Err(RubicVError::MemoryReadOutOfBounds);
        }
        Ok(i16::from_le_bytes(slice[offset..offset + 2].try_into().unwrap()))
    }

    /// Load Half-word Unsigned
    #[inline(always)]
    pub fn read_u16(&self, addr: u32) -> Result<u16, RubicVError> {
        if addr % 2 != 0 {
            return Err(RubicVError::MemoryMisaligned);
        }
        let (slice, offset) = self.is_addr_valid_for_reads(addr)?;
        if offset + 2 > slice.len() {
            return Err(RubicVError::MemoryReadOutOfBounds);
        }
        Ok(u16::from_le_bytes(slice[offset..offset + 2].try_into().unwrap()))
    }

    /// Load Word
    #[inline(always)]
    pub fn read_u32(&self, addr: u32) -> Result<u32, RubicVError> {
        if addr % 4 != 0 {
            return Err(RubicVError::MemoryMisaligned);
        }
        let (slice, offset) = self.is_addr_valid_for_reads(addr)?;
        if offset + 4 > slice.len() {
            return Err(RubicVError::MemoryReadOutOfBounds);
        }
        Ok(u32::from_le_bytes(slice[offset..offset + 4].try_into().unwrap()))
    }

    /// Store Byte
    #[inline(always)]
    pub fn write_u8(&mut self, addr: u32, value: u8) -> Result<(), RubicVError> {
        let (slice, offset) = self.is_addr_valid_for_writes(addr)?;
        unsafe {
            if let Some(target) = (*slice).get_mut(offset) {
                *target = value;
                Ok(())
            } else {
                Err(RubicVError::MemoryWriteOutOfBounds)
            }
        }
    }

    /// Store Half-word
    #[inline(always)]
    pub fn write_u16(&mut self, addr: u32, value: u16) -> Result<(), RubicVError> {
        if addr % 2 != 0 {
            return Err(RubicVError::MemoryMisaligned);
        }
        let (slice, offset) = self.is_addr_valid_for_writes(addr)?;
        unsafe {
            let slice = &mut (*slice);
            if offset + 2 > slice.len() {
                return Err(RubicVError::MemoryWriteOutOfBounds);
            }
            slice[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
            Ok(())
        }
    }

    /// Store Word
    #[inline(always)]
    pub fn write_u32(&mut self, addr: u32, value: u32) -> Result<(), RubicVError> {
        // println!("write u32 {} {}", addr, value);
        if addr % 4 != 0 {
            return Err(RubicVError::MemoryMisaligned);
        }
        let (slice, offset) = self.is_addr_valid_for_writes(addr)?;
        unsafe {
            let slice = &mut (*slice);
            if offset + 4 > slice.len() {
                return Err(RubicVError::MemoryWriteOutOfBounds);
            }
            slice[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            Ok(())
        }
    }

    pub fn step(&mut self) -> Result<(), RubicVError> {
        // println!("PC: 0x{:08x}", self.pc);
        let word = self.read_u32(self.pc)?;
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
                let byte = self.read_u8(addr)?;
                sign_extend(byte as u32, 8)
            }
            InsnKind::LH => {
                let half = self.read_u16(addr)?;
                sign_extend(half as u32, 16)
            }
            InsnKind::LW => {
                self.read_u32(addr)?
            }
            InsnKind::LBU => {
                self.read_u8(addr)? as u32
            }
            InsnKind::LHU => {
                self.read_u16(addr)? as u32
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
                self.write_u8(addr, rs2 as u8)?;
            }
            InsnKind::SH => {
                self.write_u16(addr, rs2 as u16)?;
            }
            InsnKind::SW => {
                self.write_u32(addr, rs2)?;
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
                Err(RubicVError::SystemCall(self.registers[10]))
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
        self.registers[2] = RW_STACK_START;

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

