#[derive(Debug)]
pub enum RubiconVError {
    IllegalInstruction,
    MemoryReadOutOfBounds,
    MemoryWriteOutOfBounds,
    MisalignedAccess,
    IllegalMemoryAccess,
    WriteToReadOnlyMemory,
}