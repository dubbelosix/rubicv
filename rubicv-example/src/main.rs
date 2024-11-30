#![no_std]
#![no_main]

use rubicv_sdk::*;

fn start(args: &[u32], _readonly: &[u32], scratch: &mut [u32]) {
    let n = args[0];
    let mut sum = 0;

    for i in 0..n {
        sum += i;
    }

    scratch[0] = sum;
}

entrypoint!(start);

