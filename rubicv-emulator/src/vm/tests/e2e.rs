use crate::instructions::predecode;
use super::*;

#[test]
fn test_sum_program() {
    let code = include_bytes!("test_data/sum_n.bin");
    let predecoded_program = predecode(code, CODE_START);

    let mut memory = setup_memory();

    let num_iterations = 10000u32;

    let args = [num_iterations];
    let ro_mem_start = RO_START as usize;
    // args are copied into the readonly args section
    memory.memory_slab[ro_mem_start..ro_mem_start+4].copy_from_slice(&args[0].to_le_bytes());

    // Create VM instance
    let mut vm = VMType::new(
        predecoded_program.writes_to_x0,
        memory.memory_slab.as_mut() as *mut [u8],
        &predecoded_program.instructions
    );

    // Run until completion (should hit ecall)
    match vm.as_operations().run(args.len() as u32, Some(100000)) {
        ExecutionResult::Success(result) => {
            // Check the result in a0 (x10)
            assert_eq!(result, 0);
            let value = vm.as_operations().read_u32(0x00002000);
            assert_eq!(value, (num_iterations-1)*(num_iterations)/2);
        },
        other => panic!("Unexpected execution result: {:?}", other),
    }
}