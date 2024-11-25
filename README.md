# RubicV

A custom RISC-V virtual machine implementation.

## Attribution

Parts of this implementation are derived from [RISC0](https://github.com/risc0/risc0), specifically the isa, instruction decoder and compute execution. RISC0 is licensed under the Apache License, Version 2.0.

## Memory Layout

RubicV expects a specific memory layout defined in `linker/link.x`. Programs built to run on RubicV must use this linker script.