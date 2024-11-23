#[derive(Debug)]
pub enum RubicVError {
    IllegalInstruction,
    MemoryReadOutOfBounds,
    MemoryWriteOutOfBounds,
    MisalignedAccess,
    IllegalMemoryAccess,
    WriteToReadOnlyMemory,
}