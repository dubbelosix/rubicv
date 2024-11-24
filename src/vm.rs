use crate::instructions::{FastDecodeTable, InsnKind};
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
    bss_memory_ptr: WriteSlice,
    rw_slab: WriteSlice,

    ro_code_maxsize: usize,
    rw_heap_maxsize: usize,
    rw_stack_maxsize: usize,

    ro_slab_maxsize: usize,
    rw_slab_maxsize: usize,

    // Precomputes
    rw_heap_end: u32,
    rw_stack_start: u32,
    rw_stack_end: u32,
    rw_slab_end: u32,
    ro_code_end: u32,
    ro_slab_end: u32,

    fast_decode_table: FastDecodeTable,
}

impl VM<'_> {
    pub fn new<'a>(code_memory: &'a [u8],
               ro_slab: &'a [u8],
               bss_memory_ptr: *mut [u8],
               rw_slab: *mut [u8],
               rw_heap_maxsize: usize,
               rw_stack_maxsize: usize,
               ro_code_maxsize: usize,
               ro_slab_maxsize: usize,
               rw_slab_maxsize: usize,

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

        VM {
            registers: [0;32],
            pc: 0,
            cycle_count: 0,
            code_memory,
            bss_memory_ptr,
            rw_slab,
            ro_slab,
            rw_heap_maxsize,
            rw_stack_maxsize,
            ro_code_maxsize,
            ro_slab_maxsize,
            rw_slab_maxsize,
            rw_heap_end,
            rw_stack_start,
            rw_stack_end,
            rw_slab_end,
            ro_code_end,
            ro_slab_end,

            fast_decode_table: FastDecodeTable::default()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    const TEST_MEM_SIZE: usize = 1024; // 1KB for each region

    struct TestMemory {
        bss_memory: [u8; TEST_MEM_SIZE*2],
        rw_slab: [u8; TEST_MEM_SIZE],
        code_memory: [u8; TEST_MEM_SIZE],
        ro_slab: [u8; TEST_MEM_SIZE]
    }

    // First setup function creates and returns test memory
    fn setup_memory() -> TestMemory {
        let mut bss_memory = [0u8; TEST_MEM_SIZE*2];
        let mut rw_slab = [0u8; TEST_MEM_SIZE];
        let mut code_memory =  [0u8; TEST_MEM_SIZE];
        let ro_slab = [0u8; TEST_MEM_SIZE];

        // Set some recognizable patterns
        for i in 0..TEST_MEM_SIZE*2 {
            bss_memory[i] = ((i + 1) % 256) as u8;
        }

        for i in 0..TEST_MEM_SIZE {
            rw_slab[i] = ((i + 2) % 256) as u8;
            code_memory[i] =  (i % 256) as u8;
        }

        TestMemory {
            bss_memory,
            rw_slab,
            code_memory,
            ro_slab,
        }
    }

    fn setup_vm(memory: &mut TestMemory) -> VM {
        VM::new(
            &memory.code_memory,
            &memory.ro_slab,
            &mut memory.bss_memory as *mut [u8],
            &mut memory.rw_slab as *mut [u8],
            TEST_MEM_SIZE,    // heap size
            TEST_MEM_SIZE,    // stack size
            TEST_MEM_SIZE,    // code size
            TEST_MEM_SIZE,    // ro slab size
            TEST_MEM_SIZE,    // rw slab size
        )
    }

    #[test]
    fn test_heap_and_stack_separation() {
        let mut memory = setup_memory();
        let mut vm = setup_vm(&mut memory);

        // Test heap (grows upward from RW_HEAP_START)
        assert!(vm.write_u32(RW_HEAP_START, 0xDEADBEEF).is_ok());
        assert_eq!(vm.read_u32(RW_HEAP_START), Ok(0xDEADBEEF));

        // Test stack (grows downward from RW_STACK_START)
        let stack_top = RW_STACK_START - 4;
        assert!(vm.write_u32(stack_top, 0xCAFEBABE).is_ok());
        assert_eq!(vm.read_u32(stack_top), Ok(0xCAFEBABE));

        // Verify heap and stack don't interfere
        assert_eq!(vm.read_u32(RW_HEAP_START), Ok(0xDEADBEEF));
        assert_eq!(vm.read_u32(stack_top), Ok(0xCAFEBABE));
    }

    #[test]
    fn test_memory_bounds() {
        let mut memory = setup_memory();
        let mut vm = setup_vm(&mut memory);

        // Test heap bounds
        assert!(vm.write_u32(RW_HEAP_START + TEST_MEM_SIZE as u32 - 4, 0).is_ok());
        assert_eq!(
            vm.write_u32(RW_HEAP_START + TEST_MEM_SIZE as u32, 0),
            Err(RubicVError::MemoryWriteOutOfBounds)
        );

        // Test stack bounds
        let stack_bottom = RW_STACK_START - TEST_MEM_SIZE as u32;
        assert!(vm.write_u32(stack_bottom, 0).is_ok());
        assert_eq!(
            vm.write_u32(stack_bottom - 4, 0),
            Err(RubicVError::MemoryWriteOutOfBounds)
        );
    }

    #[test]
    fn test_ro_regions() {
        let mut memory = setup_memory();
        let mut vm = setup_vm(&mut memory);

        // Test code region
        assert!(vm.read_u32(RO_CODE_START).is_ok());
        assert_eq!(
            vm.write_u32(RO_CODE_START, 0),
            Err(RubicVError::MemoryWriteOutOfBounds)
        );

        // Test ro slab
        assert!(vm.read_u32(RO_CUSTOM_SLAB_START).is_ok());
        assert_eq!(
            vm.write_u32(RO_CUSTOM_SLAB_START, 0),
            Err(RubicVError::MemoryWriteOutOfBounds)
        );
    }

    #[test]
    fn test_alignment() {
        let mut memory = setup_memory();
        let mut vm = setup_vm(&mut memory);

        // Test misaligned read/write for u16
        assert_eq!(
            vm.read_u16(RW_HEAP_START + 1),
            Err(RubicVError::MemoryMisaligned)
        );
        assert_eq!(
            vm.write_u16(RW_HEAP_START + 1, 0),
            Err(RubicVError::MemoryMisaligned)
        );

        // Test misaligned read/write for u32
        assert_eq!(
            vm.read_u32(RW_HEAP_START + 2),
            Err(RubicVError::MemoryMisaligned)
        );
        assert_eq!(
            vm.write_u32(RW_HEAP_START + 2, 0),
            Err(RubicVError::MemoryMisaligned)
        );
    }

    #[test]
    fn test_invalid_regions() {
        let mut memory = setup_memory();
        let mut vm = setup_vm(&mut memory);

        // Test unmapped region
        let invalid_addr = 0x1000_0000;
        assert_eq!(
            vm.read_u32(invalid_addr),
            Err(RubicVError::MemoryReadOutOfBounds)
        );
        assert_eq!(
            vm.write_u32(invalid_addr, 0),
            Err(RubicVError::MemoryWriteOutOfBounds)
        );
    }

    #[test]
    fn test_signed_reads() {
        let mut memory = setup_memory();
        let mut vm = setup_vm(&mut memory);

        // Write and read negative values
        assert!(vm.write_u8(RW_HEAP_START, 0xFF).is_ok());
        assert_eq!(vm.read_i8(RW_HEAP_START), Ok(-1i8));

        assert!(vm.write_u16(RW_HEAP_START, 0xFF80).is_ok());
        assert_eq!(vm.read_i16(RW_HEAP_START), Ok(-128i16));
    }

    #[test]
    fn test_rw_slab() {
        let mut memory = setup_memory();
        let mut vm = setup_vm(&mut memory);

        // Test RW slab operations
        assert!(vm.write_u32(RW_CUSTOM_SLAB_START, 0xDEADBEEF).is_ok());
        assert_eq!(vm.read_u32(RW_CUSTOM_SLAB_START), Ok(0xDEADBEEF));

        // Test bounds
        assert_eq!(
            vm.write_u32(RW_CUSTOM_SLAB_START + TEST_MEM_SIZE as u32, 0),
            Err(RubicVError::MemoryWriteOutOfBounds)
        );
    }
}