use std::fs::File;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct Fileheader {
    pub compressed: bool, // True for compressed, False for decompressed
    pub original_size: u32, // Original size of the file before compression
    pub save_name: String,  // Name of the save file (ASCII string)
    pub header_end: u64,    // Location of the end of the header in the file
}

impl Fileheader {
    pub fn read_from_file(reader: &mut File) -> Result<Self, Error> {
        let mut compressed_byte = [0u8; 1];
        reader.read_exact(&mut compressed_byte)?;
        let compressed = compressed_byte[0] == 0xA4;

        let mut system_hour = [0u8; 1];
        reader.read_exact(&mut system_hour)?;
        // Ignore system_hour byte (possibly vestigial)

        let mut zeros = [0u8; 3];
        reader.read_exact(&mut zeros)?;
        // Ignore zeros (likely terminator)

        let mut save_name_buffer = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            reader.read_exact(&mut byte)?;
            if byte[0] == 0x00 {
                break;
            }
            save_name_buffer.push(byte[0]);
        }

        // Explicitly handle FromUtf8Error
        let save_name = match String::from_utf8(save_name_buffer) {
            Ok(name) => name,
            Err(err) => {
                eprintln!("Error converting save name to string: {:?}", err);
                // Return an appropriate error or handle it differently
                return Err(Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in save name"));
            }
        };

        // Store the location of the end of the header
        let header_end= if compressed {
            reader.seek(SeekFrom::Current(0))? + 4
        } else {
            reader.seek(SeekFrom::Current(0))?
        };

        let original_size = if compressed {
            // Capture the original size only if the file is recognized as compressed
            let mut original_size_bytes = [0u8; 4];
            reader.read_exact(&mut original_size_bytes)?;
            u32::from_le_bytes(original_size_bytes) // Read little-endian
        } else {
            // If the file is recognized as decompressed, get the size from the file system
            let metadata = reader.metadata()?;
            metadata.len() as u32
        };

        let mut terminator = [0u8; 1];
        reader.read_exact(&mut terminator)?;

        Ok(Fileheader {
            compressed,
            original_size,
            save_name,
            header_end,
        })
    }
}
