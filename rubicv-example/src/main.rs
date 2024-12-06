#![no_std]
#![no_main]

use rubicv_sdk::*;

fn start(args: &[u32], _memory_slab: &[u8], scratch: &mut [u32]) {
    let n = args[0];
    let mut sum = 0;

    for i in 0..n {
        sum += i;
        // Prevent LICM compiler optimization
        unsafe { core::ptr::write_volatile(&mut sum, sum); }
    }

    scratch[0] = sum;
}

entrypoint!(start);
