use crate::instructions::{PredecodedProgram};
use super::*;

#[test]
fn test_sum_program() {
    let elf_bytes = include_bytes!("test_data/output.bin");
    let predecoded_program = PredecodedProgram::new(elf_bytes).unwrap();

    let mut memory = setup_memory();

    let num_iterations = 7u32;

    let args = [num_iterations];
    let ro_mem_start = RO_START as usize;
    // args are copied into the readonly args section
    memory.memory_slab[ro_mem_start..ro_mem_start+4].copy_from_slice(&args[0].to_le_bytes());

    // Create VM instance
    let mut vm = VMType::new(
        predecoded_program.writes_to_x0,
        memory.memory_slab.as_mut() as *mut [u8],
        predecoded_program.entrypoint,
        &predecoded_program.instructions
    );

    // Run until completion (should hit ecall)
    match vm.as_operations().run(args.len() as u32, Some(100)) {
        ExecutionResult::Success(result) => {
            // Check the result in a0 (x10)
            assert_eq!(result, 0);
            let value = vm.as_operations().read_u32(0x00002000);
            assert_eq!(value, (num_iterations-1)*(num_iterations)/2);
        },
        other => panic!("Unexpected execution result: {:?}", other),
    }
}