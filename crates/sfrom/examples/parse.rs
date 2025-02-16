use sfrom::{Sfrom};

fn main() {
    let data = std::fs::read("game.sfrom").unwrap();
    
    let sfrom = Sfrom::parse(&data).unwrap();
    
    println!("Header: {:?}", sfrom.1.rom_data);
    
    // println!("ROM Data: {:?}", sfrom);
}