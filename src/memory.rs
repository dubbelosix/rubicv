// 32KB = 0x0000_8000  (mask: 0x0000_7FFF)
// 64KB = 0x0001_0000  (mask: 0x0000_FFFF)
// 128KB = 0x0002_0000 (mask: 0x0001_FFFF)
// 256KB = 0x0004_0000 (mask: 0x0003_FFFF)
// 512KB = 0x0008_0000 (mask: 0x0007_FFFF)
// 1MB = 0x0010_0000   (mask: 0x000F_FFFF)

// 2MB  = 0x0020_0000  (mask: 0x001F_FFFF)
// 4MB  = 0x0040_0000  (mask: 0x003F_FFFF)
// 8MB  = 0x0080_0000  (mask: 0x007F_FFFF)
// 16MB = 0x0100_0000  (mask: 0x00FF_FFFF)
// 32MB = 0x0200_0000  (mask: 0x01FF_FFFF)
// 64MB = 0x0400_0000  (mask: 0x03FF_FFFF)
// 128MB = 0x0800_0000 (mask: 0x07FF_FFFF)
// 256MB = 0x1000_0000 (mask: 0x0FFF_FFFF)

pub const RW_START: u32 = 0x0000_0000;
pub const RW_SIZE: u32 = 0x0001_0000;
pub const RW_MASK: u32 = RW_SIZE - 1;
pub const RO_START: u32 = 0x9000_0000;
pub const RO_MASK: u32 = RO_SIZE - 1;
pub const RO_SIZE: u32 = 0x0040_0000;

pub const CODE_START: u32 = RW_START;
pub const CODE_SIZE: u32 = 0x0000_2000;  // 8KB

pub const SCRATCH_SIZE: u32 = 256;
pub const SCRATCH_START: u32 = CODE_START + CODE_SIZE;

pub const HEAP_START: u32 = CODE_START + CODE_SIZE + SCRATCH_SIZE;

pub const STACK_START: u32 = RW_START + RW_SIZE - 4;




