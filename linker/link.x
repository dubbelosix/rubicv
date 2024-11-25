/* Memory region definitions with permissions */
MEMORY {
    RO_CODE (rx)      : ORIGIN = 0x20000000, LENGTH = 0x60000000
    RW_HEAP (rw)      : ORIGIN = 0x30000000, LENGTH = 0x0FFFFFFC
    STACK (rw)        : ORIGIN = 0x30000000, LENGTH = 0x0FFFFFFC  /* Grows down from top */
    RO_SLAB (r)       : ORIGIN = 0x80000000, LENGTH = 0x10000000
    RW_SLAB (rw)      : ORIGIN = 0x90000000, LENGTH = 0x10000000
}

/* Exported symbols for program use */
_heap_start = 0x30000000;
_stack_bottom = 0x30000000;  /* Where heap ends and minimum stack address begins */
_stack_top = 0x3FFFFFFC;     /* Initial stack pointer */
_ro_slab_start = 0x80000000;
_rw_slab_start = 0x90000000;

SECTIONS {
    /* Code section starts at RO_CODE_START */
    . = 0x20000000;
    .text : {
        *(.text.init)
        *(.text .text.*)
    } >RO_CODE

    .rodata : {
        *(.rodata .rodata.*)
    } >RO_CODE

    /* Heap starts at RW_HEAP_START */
    . = 0x30000000;
    .data : {
        *(.data .data.*)
    } >RW_HEAP

    .bss : {
        *(.bss .bss.*)
    } >RW_HEAP

    /* Stack grows down from _stack_top */
    /* Custom slabs are at their respective addresses and accessed directly */

    /* Sanity checks */
    ASSERT(SIZEOF(.text) + SIZEOF(.rodata) < 0x60000000, "Code/rodata too large!")
    ASSERT(SIZEOF(.data) + SIZEOF(.bss) < 0x0FFFFFFC, "Data/BSS too large!")
    ASSERT(_stack_top > _stack_bottom, "Stack overlaps with heap!")
}

/* Define the entry point */
ENTRY(_start)