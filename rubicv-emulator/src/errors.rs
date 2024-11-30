#[derive(Debug, PartialEq)]
pub enum RubicVError {
    IllegalInstruction,
    InvalidInstruction,
    MemoryReadOutOfBounds,
    MemoryWriteOutOfBounds,
    MemoryMisaligned,
    MisalignedAccess,
    IllegalMemoryAccess,
    WriteToReadOnlyMemory,
    Breakpoint, // :P
    ELFDecodeError,
    SystemCall(u32)
}