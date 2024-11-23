pub const REGION_INVALID: u8 = 0;
pub const REGION_RW: u8 = 1;
pub const REGION_RO: u8 = 2;

macro_rules! define_regions {
    (
        $( ($region_name:ident, $region_type:expr, $start_addr:expr) ),*
        $(,)?
    ) => {
        $(
            pub const $region_name: u32 = $start_addr;
        )*

        pub const REGION_TABLE: [u8; 16] = {
            let mut table = [REGION_INVALID; 16];
            $(
                table[($start_addr >> 28) as usize] = $region_type;
            )*
            table
        };
    }
}

define_regions!(
    (RO_CODE_START, REGION_RO, 0x2000_0000u32),
    (RW_HEAP_START, REGION_RW, 0x3000_0000u32),
    (RW_STACK_START, REGION_RW, 0x4000_0000u32),
    (RO_CUSTOM_SLAB_START, REGION_RO, 0x8000_0000u32),
    (RW_CUSTOM_SLAB_START, REGION_RW, 0x9000_0000u32),
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_constants() {
        // Verify the region start addresses are correct
        assert_eq!(RO_CODE_START, 0x2000_0000);
        assert_eq!(RW_HEAP_START, 0x3000_0000);
        assert_eq!(RW_STACK_START, 0x4000_0000);
        assert_eq!(RO_CUSTOM_SLAB_START, 0x8000_0000);
        assert_eq!(RW_CUSTOM_SLAB_START, 0x9000_0000);
    }

    #[test]
    fn test_region_table_lookup() {
        // Test code region (0x2_)
        assert_eq!(REGION_TABLE[0x2], REGION_RO);

        // Test heap region (0x3_)
        assert_eq!(REGION_TABLE[0x3], REGION_RW);

        // Test stack region (0x4_)
        assert_eq!(REGION_TABLE[0x4], REGION_RW);

        // Test custom slab regions (0x8_, 0x9_)
        assert_eq!(REGION_TABLE[0x8], REGION_RO);
        assert_eq!(REGION_TABLE[0x9], REGION_RW);
    }

    #[test]
    fn test_invalid_regions() {
        // Test some invalid regions
        assert_eq!(REGION_TABLE[0x0], REGION_INVALID); // Lower bound
        assert_eq!(REGION_TABLE[0x1], REGION_INVALID); // Before code region
        assert_eq!(REGION_TABLE[0x5], REGION_INVALID); // Between regions
        assert_eq!(REGION_TABLE[0x7], REGION_INVALID); // Between regions
        assert_eq!(REGION_TABLE[0xA], REGION_INVALID); // After last region
        assert_eq!(REGION_TABLE[0xF], REGION_INVALID); // Upper bound
    }

    #[test]
    fn test_address_resolution() {
        // Helper function to get region type for an address
        fn get_region_type(addr: u32) -> u8 {
            REGION_TABLE[(addr >> 28) as usize]
        }

        // Test start of regions
        assert_eq!(get_region_type(0x2000_0000), REGION_RO);
        assert_eq!(get_region_type(0x3000_0000), REGION_RW);
        assert_eq!(get_region_type(0x4000_0000), REGION_RW);
        assert_eq!(get_region_type(0x8000_0000), REGION_RO);
        assert_eq!(get_region_type(0x9000_0000), REGION_RW);

        // Test middle of regions
        assert_eq!(get_region_type(0x2ABC_DEAD), REGION_RO);
        assert_eq!(get_region_type(0x3FFF_FFFF), REGION_RW);
        assert_eq!(get_region_type(0x4123_4567), REGION_RW);
        assert_eq!(get_region_type(0x8765_4321), REGION_RO);
        assert_eq!(get_region_type(0x9DEF_BEEF), REGION_RW);

        // Test invalid addresses
        assert_eq!(get_region_type(0x0000_0000), REGION_INVALID);
        assert_eq!(get_region_type(0x1FFF_FFFF), REGION_INVALID);
        assert_eq!(get_region_type(0x5000_0000), REGION_INVALID);
        assert_eq!(get_region_type(0xF000_0000), REGION_INVALID);
    }

    #[test]
    fn test_region_boundaries() {
        fn get_region_type(addr: u32) -> u8 {
            REGION_TABLE[(addr >> 28) as usize]
        }

        // Test region transitions
        assert_eq!(get_region_type(0x1FFF_FFFF), REGION_INVALID); // Just before code
        assert_eq!(get_region_type(0x2000_0000), REGION_RO);      // Start of code
        assert_eq!(get_region_type(0x2FFF_FFFF), REGION_RO);      // End of code
        assert_eq!(get_region_type(0x3000_0000), REGION_RW);      // Start of heap

        // Test upper boundaries
        assert_eq!(get_region_type(0x9FFF_FFFF), REGION_RW);      // End of last RW region
        assert_eq!(get_region_type(0xA000_0000), REGION_INVALID); // After last region
    }
}