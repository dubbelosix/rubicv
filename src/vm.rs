use crate::instructions::FastDecodeTable;
use crate::errors::RubiconVError;
use crate::memory_bounds::*;

pub struct AddrRangePrecomputes {

}

pub struct VM<'a> {
    registers: [u32; 32],
    pc: u32,
    cycle_count: usize,

    code_memory: &'a [u8],
    ro_slab: &'static [u8],
    bss_memory_ptr: *mut [u8],
    rw_slab: *mut [u8],

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
               ro_slab: &'static [u8],
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
    fn is_addr_valid_for_writes(&self, addr: u32) -> Result<(), RubiconVError> {
        match self.get_region_type(addr) {
            REGION_RW => {
                // Further check within the region
                if addr >= RW_HEAP_START && addr < self.rw_heap_end {
                    Ok(())
                } else if addr >= self.rw_stack_start && addr < self.rw_stack_end {
                    Ok(())
                } else if addr >= RW_CUSTOM_SLAB_START && addr < self.rw_slab_end {
                    Ok(())
                } else {
                    Err(RubiconVError::MemoryWriteOutOfBounds)
                }
            }
            _ => Err(RubiconVError::MemoryWriteOutOfBounds),
        }
    }

    #[inline(always)]
    fn is_addr_valid_for_reads(&self, addr: u32) -> Result<(), RubiconVError> {
        match self.get_region_type(addr) {
            REGION_RW => self.is_addr_valid_for_writes(addr),
            REGION_RO => {
                if addr >= RO_CODE_START && addr < self.ro_code_end {
                    Ok(())
                } else if addr >= RO_CUSTOM_SLAB_START && addr < self.ro_slab_end {
                    Ok(())
                } else {
                    Err(RubiconVError::MemoryReadOutOfBounds)
                }
            }
            _ => Err(RubiconVError::MemoryReadOutOfBounds),
        }
    }

    // fn load_byte(&self, addr: u32) -> Result<u8, RubiconVError> {
    //     if addr >= DATA_START && addr < DATA_START + self.data_memory.len() as u32 {
    //         // Accessing data slab
    //         let offset = (addr - DATA_START) as usize;
    //         Ok(self.data_memory[offset])
    //     } else if addr >= OUTPUT_START && addr < OUTPUT_START + self.output_buffer.len() as u32 {
    //         // Accessing output buffer
    //         let offset = (addr - OUTPUT_START) as usize;
    //         Ok(self.output_buffer[offset])
    //     } else if addr >= CODE_START && addr < CODE_START + self.code_memory.len() as u32 {
    //         // Accessing code memory (if you want to allow reading code memory)
    //         let offset = (addr - CODE_START) as usize;
    //         Ok(self.code_memory[offset])
    //     } else {
    //         Err(RubiconVError::MemoryAccessOutOfBoundsLoad)
    //     }
    // }

}