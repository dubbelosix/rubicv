use super::*;

const TEST_MEM_SIZE: usize = 1024;
const SMALL_TEST_MEM_SIZE: usize = 64;

struct TestMemory {
    bss_memory: [u8; TEST_MEM_SIZE*2],
    rw_slab: [u8; TEST_MEM_SIZE],
    code_memory: [u8; TEST_MEM_SIZE],
    ro_slab: [u8; TEST_MEM_SIZE]
}
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

// Helper to create encoded load instruction
fn encode_load(rd: u32, rs1: u32, func3: u32, imm: i32) -> u32 {
    let opcode = 0x03;  // Load opcode
    let imm = (imm as u32) & 0xFFF; // 12-bit immediate
    (imm << 20) | (rs1 << 15) | (func3 << 12) | (rd << 7) | opcode
}

// Helper to create encoded store instruction
fn encode_store(rs2: u32, rs1: u32, func3: u32, imm: i32) -> u32 {
    let opcode = 0x23;  // Store opcode
    let imm = (imm as u32) & 0xFFF; // 12-bit immediate
    let imm115 = (imm >> 5) & 0x7F;
    let imm40 = imm & 0x1F;
    (imm115 << 25) | (rs2 << 20) | (rs1 << 15) | (func3 << 12) | (imm40 << 7) | opcode
}

#[test]
fn test_load_byte() {
    let mut code_memory = [0u8; TEST_MEM_SIZE];

    // Create instruction: LB x1, 0(x2) - Load byte from address in x2
    let instruction = encode_load(1, 2, 0x0, 0); // Use x2 as base, offset 0
    code_memory[0..4].copy_from_slice(&instruction.to_le_bytes());

    let mut bss_memory = [0u8; TEST_MEM_SIZE * 2];

    // Test case 1: Sign bit 1
    bss_memory[0] = 0xFF;  // 1111_1111, sign bit is 1
    let mut vm = VM::new(
        &code_memory,
        &[0u8; TEST_MEM_SIZE],
        &mut bss_memory as *mut [u8],
        &mut [0u8; TEST_MEM_SIZE] as *mut [u8],
        TEST_MEM_SIZE,
        TEST_MEM_SIZE,
        TEST_MEM_SIZE,
        TEST_MEM_SIZE,
        TEST_MEM_SIZE,
    );
    vm.registers[2] = RW_HEAP_START;
    vm.step().unwrap();
    assert_eq!(vm.registers[1], 0xFFFF_FFFF); // Sign-extended with 1s

    // Test case 2: Sign bit 0
    bss_memory[0] = 0x7F;  // 0111_1111, sign bit is 0
    let mut vm = VM::new(
        &code_memory,
        &[0u8; TEST_MEM_SIZE],
        &mut bss_memory as *mut [u8],
        &mut [0u8; TEST_MEM_SIZE] as *mut [u8],
        TEST_MEM_SIZE,
        TEST_MEM_SIZE,
        TEST_MEM_SIZE,
        TEST_MEM_SIZE,
        TEST_MEM_SIZE,
    );
    vm.registers[2] = RW_HEAP_START;
    vm.step().unwrap();
    assert_eq!(vm.registers[1], 0x0000_007F); // Sign-extended with 0s
}

#[test]
fn test_store_load_sequence() {
    let store_instruction = encode_store(2, 1, 0x2, 8);
    let load_instruction = encode_load(3, 1, 0x2, 8);

    let mut code_memory = [0u8; TEST_MEM_SIZE];
    code_memory[0..4].copy_from_slice(&store_instruction.to_le_bytes());
    code_memory[4..8].copy_from_slice(&load_instruction.to_le_bytes());

    let mut vm = VM::new(
        &code_memory,
        &[0u8; TEST_MEM_SIZE],
        &mut [0u8; TEST_MEM_SIZE * 2] as *mut [u8],
        &mut [0u8; TEST_MEM_SIZE] as *mut [u8],
        TEST_MEM_SIZE,
        TEST_MEM_SIZE,
        TEST_MEM_SIZE,
        TEST_MEM_SIZE,
        TEST_MEM_SIZE,
    );

    // rs1 base address. offset 8, so we're writing to start of heap + 8
    vm.registers[1] = RW_HEAP_START;
    // rs2 value to store
    vm.registers[2] = 0xDEADBEEF;
    // destination register for load is init to 0
    vm.registers[3] = 0;

    // desired heap address contains 0 at init
    assert_eq!(vm.read_u32(RW_HEAP_START+8), Ok(0x0));
    // Execute store
    vm.step().unwrap();
    assert_eq!(vm.pc, RO_CODE_START+4);
    // desired heap address contains 0xDEADBEEF
    assert_eq!(vm.read_u32(RW_HEAP_START+8), Ok(0xDEADBEEF));
    assert_eq!(vm.registers[3], 0);

    // Execute load
    vm.step().unwrap();
    assert_eq!(vm.registers[3], 0xDEADBEEF);
    assert_eq!(vm.pc, RO_CODE_START+8);
}

#[test]
fn test_stack_operations() {
    let store_instruction = encode_store(1, 2, 0x2, -8);
    let load_instruction = encode_load(3, 2, 0x2, -8);

    let mut code_memory = [0u8; TEST_MEM_SIZE];
    code_memory[0..4].copy_from_slice(&store_instruction.to_le_bytes());
    code_memory[4..8].copy_from_slice(&load_instruction.to_le_bytes());

    let mut vm = VM::new(
        &code_memory,
        &[0u8; TEST_MEM_SIZE],
        &mut [0u8; TEST_MEM_SIZE * 2] as *mut [u8],
        &mut [0u8; TEST_MEM_SIZE] as *mut [u8],
        SMALL_TEST_MEM_SIZE,
        SMALL_TEST_MEM_SIZE,
        SMALL_TEST_MEM_SIZE,
        SMALL_TEST_MEM_SIZE,
        SMALL_TEST_MEM_SIZE,
    );

    // Set up registers
    vm.registers[1] = 0xFEEDBEEF;
    vm.registers[2] = RW_STACK_START;
    vm.registers[3] = 0;

    assert_eq!(vm.read_u32(RW_STACK_START-8), Ok(0x0));

    // Execute store
    vm.step().unwrap();
    assert_eq!(vm.pc, RO_CODE_START+4);
    assert_eq!(vm.registers[3], 0);
    assert_eq!(vm.read_u32(RW_STACK_START-8), Ok(0xFEEDBEEF));
    // Execute load
    vm.step().unwrap();
    assert_eq!(vm.registers[3], 0xFEEDBEEF);
    assert_eq!(vm.pc, RO_CODE_START+8);
}

#[test]
fn test_half_word_operations() {
    // Create instructions:
    // 1. SH x1, 2(x0)   - Store half
    // 2. LH x2, 2(x0)   - Load half (signed)
    // 3. LHU x3, 2(x0)  - Load half unsigned
    let store_instruction = encode_store(2, 1, 0x1, 2);
    let load_signed = encode_load(2, 1, 0x1, 2);
    let load_unsigned = encode_load(3, 1, 0x5, 2);

    let mut code_memory = [0u8; TEST_MEM_SIZE];
    code_memory[0..4].copy_from_slice(&store_instruction.to_le_bytes());
    code_memory[4..8].copy_from_slice(&load_signed.to_le_bytes());
    code_memory[8..12].copy_from_slice(&load_unsigned.to_le_bytes());

    let mut vm = VM::new(
        &code_memory,
        &[0u8; TEST_MEM_SIZE],
        &mut [0u8; TEST_MEM_SIZE * 2] as *mut [u8],
        &mut [0u8; TEST_MEM_SIZE] as *mut [u8],
        SMALL_TEST_MEM_SIZE,
        SMALL_TEST_MEM_SIZE,
        SMALL_TEST_MEM_SIZE,
        SMALL_TEST_MEM_SIZE,
        SMALL_TEST_MEM_SIZE,
    );

    // Set up register with a negative number when interpreted as i16
    vm.registers[2] = 0x8000;
    // Heap Address
    vm.registers[1] = RW_HEAP_START;

    // Execute all instructions
    vm.step().unwrap();  // store
    vm.step().unwrap();  // load signed
    vm.step().unwrap();  // load unsigned

    assert_eq!(vm.registers[2], 0xFFFF_8000);  // Sign-extended
    assert_eq!(vm.registers[3], 0x0000_8000);  // Zero-extended
    assert_eq!(vm.pc, RO_CODE_START+12);
}