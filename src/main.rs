use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() {
    let mut file = File::open("A40A8793.CR2").expect("ファイルが開けません");
    let mut header = [0u8; 8];
    file.read_exact(&mut header).unwrap();
    println!("{:x?}", header);

    let le = match &header[0..2] {
        b"II" => true,
        b"MM" => false,
        _ => panic!("不明なエンディアン"),
    };
    let ifd_offset = if le {
        u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize
    } else {
        u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize
    };
    println!("IFDオフセット: {}", ifd_offset);
    file.seek(SeekFrom::Start(ifd_offset as u64)).unwrap();

    let mut ifd_entry_count_bytes = [0u8; 2];
    file.read_exact(&mut ifd_entry_count_bytes).unwrap();
    let ifd_entry_count = if le {
        u16::from_le_bytes(ifd_entry_count_bytes)
    } else {
        u16::from_be_bytes(ifd_entry_count_bytes)
    };
    println!("IFDエントリ数: {}", ifd_entry_count);

    for i in 0..ifd_entry_count {
        let mut entry_bytes = [0u8; 12];
        file.read_exact(&mut entry_bytes).unwrap();

        let tag = if le {
            u16::from_le_bytes([entry_bytes[0], entry_bytes[1]])
        } else {
            u16::from_be_bytes([entry_bytes[0], entry_bytes[1]])
        };
        let field_type = if le {
            u16::from_le_bytes([entry_bytes[2], entry_bytes[3]])
        } else {
            u16::from_be_bytes([entry_bytes[2], entry_bytes[3]])
        };
        let count = if le {
            u32::from_le_bytes([
                entry_bytes[4],
                entry_bytes[5],
                entry_bytes[6],
                entry_bytes[7],
            ])
        } else {
            u32::from_be_bytes([
                entry_bytes[4],
                entry_bytes[5],
                entry_bytes[6],
                entry_bytes[7],
            ])
        };
        let value_offset = if le {
            u32::from_le_bytes([
                entry_bytes[8],
                entry_bytes[9],
                entry_bytes[10],
                entry_bytes[11],
            ])
        } else {
            u32::from_be_bytes([
                entry_bytes[8],
                entry_bytes[9],
                entry_bytes[10],
                entry_bytes[11],
            ])
        };

        println!(
            "エントリ {}: タグ=0x{:04x}, タイプ={}, カウント={}, 値/オフセット={}",
            i + 1,
            tag,
            field_type,
            count,
            value_offset
        );
    }
}
