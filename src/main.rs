use std::fs::File;
use std::io::Read;

fn main() {
    let mut file = File::open("/Users/takahashirikimaru/develop/rust/photapp/A40A8793.CR2")
        .expect("ファイルが開けません");
    let mut header = [0u8; 8];
    file.read_exact(&mut header).unwrap();
    println!("{:x?}", header);
}
