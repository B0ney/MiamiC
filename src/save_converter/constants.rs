use std::ops::{Range, RangeInclusive};

pub const FILE_SIZE: usize = 0x31464;
pub const CHKSM_RANGE: Range<usize> = 0x31460..0x31464;     // Provides both start and end indexes of where the file checksum is stored.
pub const SAVE_DATA: RangeInclusive<usize> = 0..=0x3145F;   // Provides both start and end indexes of where the save data is stored.
