use crate::fileheader::Fileheader;
use std::env;
use std::fs::{File};
use std::io::{self, Read, Seek, SeekFrom, Write};

mod compression;
mod decompression;
mod fileheader;

fn main() -> io::Result<()> {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if the number of arguments is correct
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        return Ok(());
    }

    // Extract input and output file paths
    let input_path = &args[1];
    let output_path = &args[2];

    // Open the input file
    let mut file = File::open(input_path)?;

    // Read file header
    let header = Fileheader::read_from_file(&mut file)?;
    println!("{:?}", header);

    // Check if the file is compressed and collect necessary data from the header
    let is_compressed = header.compressed;
    let header_end = header.header_end;
    println!("Is it compressed: {:?}", is_compressed);
    println!("Byte the header ends at: {:?}", header_end);

    // Get original file size
    let original_file_size = if is_compressed {
        header.original_size
    } else {
        // If the file is decompressed, get the size from the file system
        let metadata = file.metadata()?;
        metadata.len() as u32
    };
    println!("File's original size in bytes: {:?}", original_file_size);

    // Seek to the end of the header
    file.seek(SeekFrom::Start(header_end))?;

    // Prepare a vector to hold the data
    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data)?;

    // Prepare output data
    let output_data = if is_compressed {
        // Decompress the data
        decompression::decompress(&file_data, header.original_size as usize)
    } else {
        // Compress the data
        compression::compress(&file_data)
    };

    // Setup new header
    // Seek to the start of the file
    file.seek(SeekFrom::Start(0))?;

    // Read header data up to the header_end position into new_header
    let mut new_header = Vec::new();
    let mut file_cursor = file.take(header_end);
    file_cursor.read_to_end(&mut new_header)?;

    if is_compressed {
        // Decompressing: Set the first byte to 0x24 and remove the last 4 bytes from the header data
        new_header[0] = 0x24;
        new_header.truncate(new_header.len() - 4);
    } else {
        // Compressing: Append original file size to the end of the header data
        let original_size_bytes = original_file_size.to_le_bytes();
        new_header.extend_from_slice(&original_size_bytes);
        // Set the first byte as a marker for compressed data
        new_header[0] = 0xA4;
    }

    // Write the decompressed or compressed data to the output file
    let mut output_file = File::create(output_path)?;
    // Write header
    output_file.write_all(&new_header)?;
    // Write processed data
    output_file.write_all(&output_data)?;

    println!("File has been processed and saved to {}", output_path);

    Ok(())
}
