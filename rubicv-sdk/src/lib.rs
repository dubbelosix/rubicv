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
            let arg_count = unsafe {
                *(ARGS_START as *const u32)
            };

            let args = unsafe {
                core::slice::from_raw_parts(
                    (ARGS_START as *const u32).add(1),  // Start reading args after the count
                    arg_count as usize
                )
            };

            let memory_slab = unsafe {
                core::slice::from_raw_parts(
                    MEMORY_START as *const u8,
                    MEMORY_SIZE as usize
                )
            };

            let scratch = unsafe {
                core::slice::from_raw_parts_mut(
                    SCRATCH_START as *mut u32,
                    SCRATCH_SIZE as usize
                )
            };

            $func(args, memory_slab, scratch);

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

#[macro_export]
macro_rules! entrypoint2 {
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
            let arg_count = unsafe {
                *(ARGS_START as *const u32)
            };

            let args = unsafe {
                core::slice::from_raw_parts(
                    (ARGS_START as *const u32).add(1),  // Start reading args after the count
                    arg_count as usize
                )
            };

            let arg0 = unsafe {
                *((ARGS_START as *const u32).add(1))  // Add 1 to move 4 bytes forward
            };

            let arg1 = unsafe {
                *((ARGS_START as *const u32).add(2))  // Add 2 to move 8 bytes forward
            };

            let arg2 = unsafe {
                *((ARGS_START as *const u32).add(3))  // Add 3 to move 12 bytes forward
            };

            let arg3 = unsafe {
                *((ARGS_START as *const u32).add(4))  // Add 4 to move 16 bytes forward
            };



            let scratch = unsafe {
                core::slice::from_raw_parts_mut(
                    SCRATCH_START as *mut u32,
                    SCRATCH_SIZE as usize
                )
            };

            $func(arg_count, args, arg0, arg1,arg2,arg3, scratch);

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
