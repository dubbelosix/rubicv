use super::*;
mod memory;
mod compute;
mod e2e;
// mod pre_decode;

use alloc::vec;
use alloc::vec::Vec;
use alloc::boxed::Box;

struct TestMemory {
    memory_slab: Box<[u8]>, // One-time 4MB heap allocation, fixed size
}

fn setup_memory() -> TestMemory {
    // 4MB on heap one time, Box for fixed-size
    let memory_slab = vec![0u8; MEMORY_SIZE as usize].into_boxed_slice();

    TestMemory {
        memory_slab
    }
}