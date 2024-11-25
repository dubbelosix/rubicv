use crate::vm::{ExecutionResult, VM};

#[test]
fn test_add_program() {
    // Read the program binary

    // Code is simply rust compiled down to riscv that does:
    // let x = args[0];
    // let y = args[1];
    // let result = x+y;

    let code = include_bytes!("test_data/output.bin");

    // Create small test memory regions
    let mut bss_memory = [0u8; 1024];  // For heap/stack
    let mut rw_slab = [0u8; 64];      // RW slab isn't used in this program
    let mut ro_args = [0u8; 64];

    let args = [42u32, 27u32];  // add is 69
    // args are copied into the readonly args section
    ro_args[..4].copy_from_slice(&args[0].to_le_bytes());
    ro_args[4..8].copy_from_slice(&args[1].to_le_bytes());

    // Create VM instance
    let mut vm = VM::new(
        code,                          // Program code from binary
        &[],                    // Empty RO slab (not used)
        &mut bss_memory as *mut [u8],
        &mut rw_slab as *mut [u8],
        &ro_args,
        512,                   // Heap size
        512,                   // Stack size
        code.len(),                           // Code size
        0,                      // No RO slab
        64,                     // RW slab size
        8                       // RO args size
    );


    // Run until completion (should hit ecall)
    match vm.run(args.len() as u32, Some(1000)) {  // Limit to 1000 cycles for safety
        ExecutionResult::Success(result) => {
            // Check the result in a0 (x10)
            assert_eq!(result, 69, "Expected 42 + 27 = 69");
        },
        other => panic!("Unexpected execution result: {:?}", other),
    }
}