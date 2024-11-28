use super::*;

fn setup_vm(memory: &mut TestMemory) -> VM<EnforceZero> {
    VM::<EnforceZero>::new(
        memory.memory_slab.as_mut() as *mut [u8],
        &[]
    )
}

#[test]
fn test_rw_read_write_u8() {
    let mut memory = setup_memory();
    let mut vm = setup_vm(&mut memory);

    // Test write and read at start of RW region
    vm.write_u8(0x0, 0xAA);
    assert_eq!(vm.read_u8(0x0), 0xAA);

    // Test write and read at end of RW region
    vm.write_u8(RW_SIZE - 1, 0xBB);
    assert_eq!(vm.read_u8(RW_SIZE - 1), 0xBB);

    // Test wraparound
    vm.write_u8(RW_SIZE, 0xCC);  // Should wrap to 0x0
    assert_eq!(vm.read_u8(0x0), 0xCC);
}

#[test]
fn test_ro_read() {
    let mut memory = setup_memory();
    let mem_begin = memory.memory_slab[RO_START as usize];
    let mem_end = memory.memory_slab[(RO_START+RO_SIZE) as usize - 1];
    let vm = setup_vm(&mut memory);

    // Test read from RO region
    let ro_value = vm.read_u8(RO_START);
    assert_eq!(ro_value, mem_begin);

    // Test read from end of RO region
    let ro_end_value = vm.read_u8(RO_START + RO_SIZE - 1);
    assert_eq!(ro_end_value, mem_end);
}

#[test]
fn test_multi_byte_operations() {
    let mut memory = setup_memory();
    let mut vm = setup_vm(&mut memory);

    // Test u16 operations
    vm.write_u16(0x0, 0xAABB);
    assert_eq!(vm.read_u16(0x0), 0xAABB);
    assert_eq!(vm.read_u8(0x0), 0xBB);  // Check little-endian
    assert_eq!(vm.read_u8(0x1), 0xAA);

    // Test u32 operations
    vm.write_u32(0x4, 0xDEADBEEF);
    assert_eq!(vm.read_u32(0x4), 0xDEADBEEF);
    assert_eq!(vm.read_u8(0x4), 0xEF);  // Check little-endian
    assert_eq!(vm.read_u8(0x5), 0xBE);
    assert_eq!(vm.read_u8(0x6), 0xAD);
    assert_eq!(vm.read_u8(0x7), 0xDE);
}

#[test]
fn test_signed_reads() {
    let mut memory = setup_memory();
    let mut vm = setup_vm(&mut memory);

    // Test i8
    vm.write_u8(0x0, 0xFF);  // -1 in two's complement
    assert_eq!(vm.read_i8(0x0), -1i8);

    // Test i16
    vm.write_u16(0x2, 0x8000);  // Minimum i16 value
    assert_eq!(vm.read_i16(0x2), i16::MIN);
}

#[test]
fn test_aligned_write_read() {
    let mut memory = setup_memory();
    let mut vm = setup_vm(&mut memory);

    // Write u32 at aligned address without wraparound
    vm.write_u32(RW_SIZE - 4, 0xAABBCCDD);
    assert_eq!(vm.read_u8(RW_SIZE - 4), 0xDD);
    assert_eq!(vm.read_u8(RW_SIZE - 3), 0xCC);
    assert_eq!(vm.read_u8(RW_SIZE - 2), 0xBB);
    assert_eq!(vm.read_u8(RW_SIZE - 1), 0xAA);
}

#[test]
fn test_ro_region_integrity() {
    let mut memory = setup_memory();
    let mut vm = setup_vm(&mut memory);

    // Save original RO values
    let original_values: Vec<u8> = (0..4)
        .map(|i| vm.read_u8(RO_START + i))
        .collect();

    // Attempt to write to RO region (should wrap in RW region)
    vm.write_u32(RO_START, 0xDEADBEEF);

    // Verify RO region is unchanged
    for i in 0..4 {
        assert_eq!(vm.read_u8(RO_START + i as u32), original_values[i]);
    }
}