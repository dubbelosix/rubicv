use super::*;
mod memory;
mod compute;
mod e2e;
mod pre_decode;


struct TestMemory {
    ro_slab: Box<[u8]>,                 // One-time 4MB heap allocation, fixed size
    rw_slab: [u8; RW_SIZE as usize],    // 64KB on stack, truly static
}

fn setup_memory() -> TestMemory {
    // stack 64KB
    let mut rw_slab = [0u8; RW_SIZE as usize];
    // 4MB on heap one time, Box for fixed-size
    let mut ro_slab = vec![0u8; RO_SIZE as usize].into_boxed_slice();

    TestMemory {
        rw_slab,
        ro_slab,
    }
}