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
            let args_ptr = ARGS_START as *const u32;
            let memory_ptr = MEMORY_START as *const u8;
            let scratch_ptr = SCRATCH_START as *mut u32;
            $func(
                args_ptr,
                memory_ptr,
                scratch_ptr,
            );

            // Return after function completes
            loop {
                unsafe {
                    core::arch::asm!(
                        "mv a1, {0}
                         ecall",
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


