use std::fs::File;
use std::io::{Read,Write};
use std::ops::{Range, RangeInclusive};
use byteorder::{BigEndian, ByteOrder, LittleEndian};

use super::enums::SaveType;
use super::constants::{
    CHKSM_RANGE,
    SAVE_DATA,
};
use super::libs::{
    Block,
    open_save_file,
    generate_block_info,
    find_save_type,
    calculate_checksum,
};

#[derive(Debug)]
pub struct SaveFile {
    pub Checksum: u32,
    pub SaveType: SaveType,
    pub File: Vec<u8>,
    pub Blocks: [Block; 23], // Array containing location for all 23 blocks
    pub save_location: String,
}

impl SaveFile {
    //NOTE: A GTA VC save file uses Little endian for binary data
    pub fn new(save_path: &str) -> Result<SaveFile, String> {       
        match open_save_file(save_path) {
            Ok(file_buffer) => {
                let save_file       = file_buffer; 
                let block_info      = generate_block_info(&save_file);
                let save_type       = find_save_type(&save_file, &block_info);
                let save_checksum   = calculate_checksum(&save_file[SAVE_DATA]);
                let save_location   = save_path.to_string();

                Ok(SaveFile {
                    File: save_file,
                    SaveType: save_type,
                    Checksum: save_checksum,
                    Blocks: block_info,
                    save_location,
                })
            },
            Err(msg) => Err(format!("{}", msg))?,
        }
    }

    fn overwrite_checksum(&mut self, new_checksum: u32) {
        // CHKSM_RANGE provides both start and end indexes of where the file checksum is stored.            
        BigEndian::write_u32(&mut self.File[CHKSM_RANGE], new_checksum);
    }

    pub fn export(&mut self, path: &str) -> Result<(),String> { 
        let new_checksum = calculate_checksum(&self.File[SAVE_DATA]);
        let mut file = File::create(format!("{}", path)).map_err(|e| e.to_string())?;

        self.overwrite_checksum(new_checksum);
        file.write_all(&self.File).map_err(|e| e.to_string())?;

        println!("Exported successfully to: {}", path);
        Ok(())
    }

    pub fn convert_save(&mut self, path: &str) -> Result<(), String> {
        /* Converts Retail saves into Steam version and vice versa.

        To convert a Retail Save to the Steam version:
            1) Insert value "0xFD00_0000" (-3 in little endian) at offset 0x0000_0058, 
               this will shift all original bytes succeeding 0x0000_0058 by 4 bytes to the right.
            
            2) The first 2 bytes of the save represents the size of the first block (block 0), 
               this must be incremented by 4.

               Before incrementing it by 4, understand that these bytes are LittleEndian,
               so the BYTES must be reversed -> incremented -> reversed again before writing into the save file.
            
            3) The 32 bit value at offset (Block_22.index) must be *decremented* by 4, 
               *BUT* the new value must be written at offset (Block_22.end - 4). (Little Endian rules apply)

            4) Delete the last 4 bytes in the save file, that is the old checksum.
               Every VC save file must be exactly 0x31464 bytes long. 

            5) Write new checksum on the last 4 bytes of the file.


        To convert a Steam Save to the Retail version:
            1) Delete the value "0xFD00_0000" at offset 0x0000_0058, 
               this will shift all original bytes succeeding 0x0000_0058 by 4 bytes to the left.
            
            2) The first 2 bytes of the save represents the size of the first block (block 0), 
               this must be decremented by 4.

               Before decrementing it by 4, understand that these bytes are LittleEndian,
               so the BYTES must be reversed -> decrement -> reversed again before writing into the save file.
            
            3) The 32 bit value at offset (Block_22.index) must be *incremented* by 4, 
               *BUT* the new value must be written at offset (Block_22.end + 4). (Little Endian rules apply)

            4) Since the save file is 4 bytes short, add new bytes at the end.
               Every VC save file must be exactly 0x31464 bytes long. 

            5) Write new checksum on the last 4 bytes of the file.

        */
        match self.SaveType {
            SaveType::Steam => {
                // Convert Steam Version to Retail version

                self.File.remove(0x58);
                self.File.remove(0x58);
                self.File.remove(0x58);
                self.File.remove(0x58);

                // decrement block 0 size by 4 since we've removed 4 items
                let new_block_0_size: u16 = (self.Blocks[0].size - 4) as u16; 

                LittleEndian::write_u16_into(&[new_block_0_size], &mut self.File[0x00..=0x01]);
                
                // increment last block size by 4
                
                let last_block_index = self.Blocks[22].end - 4;
                let last_block_range = last_block_index..(last_block_index + 4);
                let last_block_size  = LittleEndian::read_u32(&self.File[last_block_range.clone()]) + 4; // increment by 4

                LittleEndian::write_u32_into(&[last_block_size], &mut self.File[last_block_range]);

                // Since we've removed 4 items (4 bytes), we pad the save file so that we can add the new checksum.
                self.File.push(0x00);
                self.File.push(0x00);
                self.File.push(0x00);
                self.File.push(0x00);

                println!("Converted save to Retail");

            },
            SaveType::Retail => {
                // Convert Retail Version to Steam version

                // on untoched saves FD C2 F5 3D
                self.File.insert(0x58, 0xFD); // FD
                self.File.insert(0x59, 0x00); // C2
                self.File.insert(0x5A, 0x00); // F5
                self.File.insert(0x5B, 0x00); // 3D

                // increment block 0 size by 4 since we've added 4 items
                let new_block_0_size: u16 = (self.Blocks[0].size + 4) as u16;

                LittleEndian::write_u16_into(&[new_block_0_size], &mut self.File[0x00..=0x01]);

                // Decrement the last block size by 4 
                let last_block_index = self.Blocks[22].end + 4;
                let last_block_range = last_block_index..(last_block_index + 4);
                let last_block_size  = LittleEndian::read_u32(&self.File[last_block_range.clone()]) - 4;

                LittleEndian::write_u32_into(&[last_block_size], &mut self.File[last_block_range]);

                // After we inserted 2 bytes, the checksum is conveniently pushed over the strict file limit
                // So we delete the last four items
                self.File.pop();
                self.File.pop();
                self.File.pop();
                self.File.pop();

                println!("Converted save to Steam");
            },
            _ => Err("Format not supported")?
        };

        self.export(path) 
    }
}