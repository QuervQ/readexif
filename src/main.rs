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
        let type_size = get_type_size(field_type);
        let data_size = type_size * count as usize;

        if data_size <= 4 {
            // 値が直接埋め込まれている
            println!("  → 値: {:?}", &entry_bytes[8..8 + data_size]);
            print_value(&entry_bytes[8..8 + data_size], field_type, count, le);
        } else {
            // オフセット先のデータを読み取る
            println!("  → オフセット:  {}", value_offset);

            // 現在位置を保存
            let current_pos = file.stream_position().unwrap();

            // オフセット先に移動してデータを読み取る
            file.seek(SeekFrom::Start(value_offset as u64)).unwrap();
            let mut data = vec![0u8; data_size];
            file.read_exact(&mut data).unwrap();

            println!("  → データ: {:?}", &data[..data_size.min(16)]); // 最初の16バイトのみ表示
            print_value(&data, field_type, count, le);

            // 元の位置に戻る
            file.seek(SeekFrom::Start(current_pos)).unwrap();
        }
        println!();
    }
}
fn get_type_size(field_type: u16) -> usize {
    match field_type {
        1 => 1,  // BYTE
        2 => 1,  // ASCII
        3 => 2,  // SHORT
        4 => 4,  // LONG
        5 => 8,  // RATIONAL (2つのLONG)
        6 => 1,  // SBYTE
        7 => 1,  // UNDEFINED
        8 => 2,  // SSHORT
        9 => 4,  // SLONG
        10 => 8, // SRATIONAL (2つのSLONG)
        11 => 4, // FLOAT
        12 => 8, // DOUBLE
        _ => 1,  // 不明な型はとりあえず1バイトとする
    }
}
fn print_value(data: &[u8], field_type: u16, count: u32, le: bool) {
    match field_type {
        2 => {
            // ASCII文字列
            if let Ok(s) = std::str::from_utf8(data) {
                println!("  → ASCII:  {}", s.trim_end_matches('\0'));
            }
        }
        3 => {
            // SHORT (16bit unsigned)
            if count <= 4 {
                for i in 0..count as usize {
                    if i * 2 + 1 < data.len() {
                        let val = if le {
                            u16::from_le_bytes([data[i * 2], data[i * 2 + 1]])
                        } else {
                            u16::from_be_bytes([data[i * 2], data[i * 2 + 1]])
                        };
                        println!("  → SHORT[{}]: {}", i, val);
                    }
                }
            }
        }
        4 => {
            // LONG (32bit unsigned)
            if count <= 4 {
                for i in 0..count as usize {
                    if i * 4 + 3 < data.len() {
                        let val = if le {
                            u32::from_le_bytes([
                                data[i * 4],
                                data[i * 4 + 1],
                                data[i * 4 + 2],
                                data[i * 4 + 3],
                            ])
                        } else {
                            u32::from_be_bytes([
                                data[i * 4],
                                data[i * 4 + 1],
                                data[i * 4 + 2],
                                data[i * 4 + 3],
                            ])
                        };
                        println!("  → LONG[{}]: {}", i, val);
                    }
                }
            }
        }
        5 => {
            // RATIONAL (分子/分母)
            if count <= 2 {
                for i in 0..count as usize {
                    if i * 8 + 7 < data.len() {
                        let numerator = if le {
                            u32::from_le_bytes([
                                data[i * 8],
                                data[i * 8 + 1],
                                data[i * 8 + 2],
                                data[i * 8 + 3],
                            ])
                        } else {
                            u32::from_be_bytes([
                                data[i * 8],
                                data[i * 8 + 1],
                                data[i * 8 + 2],
                                data[i * 8 + 3],
                            ])
                        };
                        let denominator = if le {
                            u32::from_le_bytes([
                                data[i * 8 + 4],
                                data[i * 8 + 5],
                                data[i * 8 + 6],
                                data[i * 8 + 7],
                            ])
                        } else {
                            u32::from_be_bytes([
                                data[i * 8 + 4],
                                data[i * 8 + 5],
                                data[i * 8 + 6],
                                data[i * 8 + 7],
                            ])
                        };
                        println!(
                            "  → RATIONAL[{}]:  {}/{} = {}",
                            i,
                            numerator,
                            denominator,
                            numerator as f64 / denominator as f64
                        );
                    }
                }
            }
        }
        _ => {}
    }
}
