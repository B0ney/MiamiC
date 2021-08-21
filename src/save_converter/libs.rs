use std::fs;
use std::ops::{Range, RangeInclusive};
use byteorder::{BigEndian, ByteOrder, LittleEndian};

use super::enums::SaveType;
use super::constants::{
    FILE_SIZE,
    CHKSM_RANGE,
    SAVE_DATA,
};

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub index: usize,   // Tells us where the data is located
    pub size: u32,      
    pub end: usize,
}

impl Block {
    pub fn new() -> Self {
        Self {
            index:  0,
            size:   0,
            end:    0,
        }
    }
}

pub fn open_save_file(save_path: &str) -> Result<Vec<u8>, String> {
    let file_metadata = fs::metadata(save_path).map_err(|e| e.to_string())?;
    
    if !file_metadata.is_file() {
        Err("Unable to read save file, path is a Directory.".to_string())?
    };   
    
    // Make sure the loaded file is the right size before we load it in memory
    let save_file_size = file_metadata.len() as usize;
    
    if save_file_size != FILE_SIZE {
        Err(format!(
            "INVALID SAVE FILE! ALL GTA:VC SAVES MUST BE 0x{:04X} BYTES LARGE.\nFILE IS: 0x{:04X} BYTES LARGE.\n\nAre you sure the loaded savefile is from GTA: Vice City?",
            FILE_SIZE,
            save_file_size
            )
        )?
    };

    let save_file = fs::read(save_path).map_err(|e| format!("Unable to read save file: {}", e))?;
    /*
    DO NOT DISABLE THESE CHECKS!

    If a save file is "corrupted" from a buggy save editor, it may be possible to recover it by overwriting the checksum.
    If you, however edited the file by hand and have no idea what you changed, then there is no easy way to recover it.

    If you're curious to know what happens if you disable this check, when the program generates the necessary block infomation (index & size),
    these values are crucial in locating the next block. An incorrect value will cause a cascading effect,
    ultimately causing the program to panic due to an out of bounds index.
    
    This possibility however is quite low.

    In most cases, blindly editing a location won't cause this program to a panic, but it may cause your game to crash.
    */

    let checksum: u32 = BigEndian::read_u32(&save_file[CHKSM_RANGE]);

    checksum_is_valid(&save_file[SAVE_DATA], checksum)
        .map_err(|calculated| format!("Save file possibly corrupted as checksum failed.\nExpected: {:08X},\nGot: {:08X},",
                checksum,
                calculated )
                )?;

    Ok(save_file)
}

pub fn checksum_is_valid(data: &[u8], checksum: u32) -> Result<(), u32> {
    let calculated = calculate_checksum(data);

    if calculated == checksum {
        Ok(()) // If checksum is satisfied, return Ok
    } else {
        Err(calculated) // If checksum fails, provied the calculated checksum
    }
}

pub fn calculate_checksum(data: &[u8]) -> u32 {
    let mut output = [0u8; 4];
    LittleEndian::write_u32(&mut output, data.iter().fold(0, |y, x| y + *x as u32));
    BigEndian::read_u32(&output)
}

pub fn generate_block_info(savefile: &[u8]) -> [Block; 23] {
    /*
    A GTAVC save file is split into 23 blocks. Each block does not have a fixed size,
    so we must find out their sizes and where they are indexed in the file.
    */
    let mut blocks = [Block::new(); 23]; // initialze array
    let mut offset: usize = 0x0000_0000; // absolute address that tells us a block's size
    let mut block_start_address: usize = 0x0000_0004; // block's absolute start address
    let mut block_end_address: usize;

    for i in 0..23 {
        let dword: Range<usize> = offset..offset + 4;
        let size = LittleEndian::read_u32(&savefile[dword]);

        blocks[i] = Block {
            index: block_start_address, // index of the block's start of data
            size,
            end: block_start_address + size as usize,
        };

        offset += blocks[i].size as usize + 4;  // This will move to the next block, add 4 cause the wiki says so..
        block_start_address = offset + 4;       // Add 4 to get the address of the block's start data
    }

    blocks
}

pub fn find_save_type(save_data: &[u8], block: &[Block]) -> SaveType {
    match &block[1].size {
        0x0708 => {
            let offset: usize = 0x0058;
            let value: u8 = save_data[offset];

            match value {
                0xE8    => SaveType::Retail,
                0xfd    => SaveType::Steam,
                _       => SaveType::Unknown,
            }
        }
        0x0764  => SaveType::Android,
        0x075c  => SaveType::IOS,
        _       => SaveType::Unknown,
    }
}
