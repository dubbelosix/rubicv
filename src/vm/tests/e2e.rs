use super::*;

#[test]
fn test_sum_program() {
    let code = include_bytes!("test_data/sum_n.bin");
    let code_len = code.len();
    let mut memory = setup_memory();

    let num_iterations = 10000u32;

    let args = [num_iterations];
    // args are copied into the readonly args section
    memory.ro_slab[..4].copy_from_slice(&args[0].to_le_bytes());
    memory.rw_slab[..code_len].copy_from_slice(code);

    // Create VM instance
    let mut vm = VM::new(
        memory.ro_slab.as_mut() as *mut [u8],
        &mut memory.rw_slab as *mut [u8],
    );

    // Run until completion (should hit ecall)
    match vm.run(args.len() as u32, Some(100000)) {
        ExecutionResult::Success(result) => unsafe {
            // Check the result in a0 (x10)
            assert_eq!(result, 0);
            let value = vm.read_u32(0x00002000);
            assert_eq!(value, (num_iterations-1)*(num_iterations)/2);
        },
        other => panic!("Unexpected execution result: {:?}", other),
    }
}