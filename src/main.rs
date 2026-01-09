use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() {
    let filepath: Vec<String> = env::args().collect();
    println!("File path: {}", filepath[1]);
    let mut file = File::open(&filepath[1]).expect("Failed to open file");
    let mut buffer = [0u8; 12];
    file.read_exact(&mut buffer)
        .expect("Failed to read exact bytes");

    println!("Read bytes: {:x?}", &buffer);

    let endian = &buffer[0..2];

    let is_le = match endian {
        [0x49, 0x49] => true,  // "II"
        [0x4D, 0x4D] => false, // "MM"
        _ => panic!("Not a TIFF file"),
    };

    println!("Is little-endian: {}", is_le);

    let ifd_offset = if is_le {
        u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]])
    } else {
        u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]])
    };

    file.seek(SeekFrom::Start(ifd_offset as u64))
        .expect("Failed to seek");
}
