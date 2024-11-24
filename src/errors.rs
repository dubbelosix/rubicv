#[derive(Debug, PartialEq)]
pub enum RubicVError {
    IllegalInstruction,
    InvalidInstruction,
    MemoryReadOutOfBounds,
    MemoryWriteOutOfBounds,
    MemoryMisaligned,
    IllegalMemoryAccess,
    WriteToReadOnlyMemory,
}