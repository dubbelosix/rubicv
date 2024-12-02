MEMORY {
    CODE (rx)           : ORIGIN = 0x00000000, LENGTH = 0x2000      /* 8KB for code */
    SCRATCH (rw)        : ORIGIN = 0x00002000, LENGTH = 0x100       /* 256 bytes scratch */
    HEAP_AND_STACK (rw) : ORIGIN = 0x00002100, LENGTH = 0x0DEB0     /* Remaining space ~56KB */
    RO_SLAB (r)        : ORIGIN = 0x00010000, LENGTH = 0x003F0000   /* 4MB read-only slab */
    FORBIDDEN          : ORIGIN = 0x0, LENGTH = 0                    /* Zero-length region to force errors */
}

/* Important addresses for the program */
_code_start = 0x00000000;    /* Start of code section */
_scratch_start = 0x00002000; /* Start of scratch space */
_heap_start = 0x00002100;    /* Start of heap (after scratch) */
_stack_top  = 0x0000FFFC;    /* Top of stack (end of RW region - 4) */

SECTIONS {
    /* Code section at beginning */
    . = 0x00000000;
    .text : {
        *(.text.init)    /* Entry point and initialization */
        *(.text .text.*) /* Code */
        *(.eh_frame)     /* Exception handling frame */
    } >CODE

    /* Force error for any static data sections by putting them in FORBIDDEN region */
    .rodata : { *(.rodata .rodata.*) } >FORBIDDEN
    .data : { *(.data .data.*) } >FORBIDDEN
    .sdata : { *(.sdata .sdata.*) } >FORBIDDEN
    .got : { *(.got .got.*) } >FORBIDDEN

    /* Scratch space section */
    . = 0x00002000;
    .scratch : {
        *(.scratch .scratch.*) /* Scratch space */
    } >SCRATCH

    /* BSS goes in heap/stack region since it's just zero-initialized */
    .bss : {
        *(.bss .bss.*)   /* Uninitialized data */
        *(COMMON)
    } >HEAP_AND_STACK

    /* Sections to discard */
    /DISCARD/ : {
        *(.comment)
        *(.note.*)
        *(.riscv.attributes)
    }

    /* Sanity checks */
    ASSERT(SIZEOF(.text) <= 0x2000, "Code section exceeds 8KB!")
    ASSERT(SIZEOF(.scratch) <= 0x100, "Scratch space exceeds 256 bytes!")
    ASSERT(SIZEOF(.bss) <= 0x0DEB0, "BSS too large for heap/stack region!")

    /* Additional check to ensure no static data sections exist */
    ASSERT(SIZEOF(.rodata) == 0, "Static read-only data (.rodata) is not allowed!")
    ASSERT(SIZEOF(.data) == 0, "Static initialized data (.data) is not allowed!")
    ASSERT(SIZEOF(.sdata) == 0, "Static small data (.sdata) is not allowed!")
    ASSERT(SIZEOF(.got) == 0, "Global offset table (.got) is not allowed!")
}

/* Entry point */
ENTRY(_start)