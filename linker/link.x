MEMORY {
   CODE_AND_RODATA (rx) : ORIGIN = 0x00000000, LENGTH = 0x2000    /* 8KB for code */
   SCRATCH (rw)        : ORIGIN = 0x00002000, LENGTH = 0x100     /* 256 bytes scratch */
   HEAP_AND_STACK (rw) : ORIGIN = 0x00002100, LENGTH = 0x0DEB0   /* Remaining space ~56KB */
   RO_SLAB (r)        : ORIGIN = 0x90000000, LENGTH = 0x00400000 /* 4MB read-only slab */
}

/* Important addresses for the program */
_code_start = 0x00000000;    /* Start of code section */
_scratch_start = 0x00002000; /* Start of scratch space */
_heap_start = 0x00002100;    /* Start of heap (after scratch) */
_stack_top  = 0x0000FFFC;    /* Top of stack (end of RW region - 4) */

SECTIONS {
   /* Code section at beginning of RW region */
   . = 0x00000000;
   .text : {
       *(.text.init)    /* Entry point and initialization */
       *(.text .text.*) /* Code */
       *(.eh_frame)     /* Exception handling frame */
       *(.rodata .rodata.*) /* Read-only data (constants) */
   } >CODE_AND_RODATA

   /* Scratch space section */
   . = 0x00002000;
   .scratch : {
       *(.scratch .scratch.*) /* Scratch space */
   } >SCRATCH

   /* Read-write data section starts after scratch */
   . = 0x00002100;
   .data : {
       *(.data .data.*) /* Initialized data */
   } >HEAP_AND_STACK

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
   ASSERT(SIZEOF(.data) + SIZEOF(.bss) <= 0x0DEB0, "Data/BSS too large for heap/stack region!")
}

/* Entry point */
ENTRY(_start)