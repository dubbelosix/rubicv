#![no_std]

pub use rubicv_emulator::memory::*;

#[macro_export]
macro_rules! entrypoint {
    ($func:ident) => {
        mod alloc_impl {
            use core::alloc::{GlobalAlloc, Layout};
            struct Allocator;
            unsafe impl GlobalAlloc for Allocator {
                unsafe fn alloc(&self, _layout: Layout) -> *mut u8 { core::ptr::null_mut() }
                unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
            }
            #[global_allocator]
            static ALLOCATOR: Allocator = Allocator;
        }

        #[no_mangle]
        pub extern "C" fn _start() -> ! {
            let arg_count: u32;
            unsafe {
                core::arch::asm!("mv {}, a0", out(reg) arg_count);
            }

            let args = unsafe {
                core::slice::from_raw_parts(
                    ARGS_START as *const u32,
                    arg_count as usize
                )
            };

            let readonly = unsafe {
                core::slice::from_raw_parts(
                    RO_SLAB_START as *const u32,
                    RO_SLAB_SIZE as usize
                )
            };

            let scratch = unsafe {
                core::slice::from_raw_parts_mut(
                    SCRATCH_START as *mut u32,
                    SCRATCH_SIZE as usize
                )
            };

            $func(args, readonly, scratch);

            // Return after function completes
            loop {
                unsafe {
                    core::arch::asm!(
                        "mv a1, {0}",
                        "ecall",
                        in(reg) 0,
                        options(noreturn)
                    );
                }
            }
        }

        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {
                unsafe {
                    core::arch::asm!(
                        "mv a1, {0}",
                        "ecall",
                        in(reg) 1,
                        options(noreturn)
                    );
                }
            }
        }
    };
}
