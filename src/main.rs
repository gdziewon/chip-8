use chip8::{Chip8, Memory};
use minifb::Key;
use std::process;
use std::env;

fn main() {
    let mut chip8 = Chip8::new();

    chip8.set_colors(0x800080, 0xffc0cb); // purple and pink
    chip8.insert_binding(0x2, Key::W);
    chip8.insert_binding(0x4, Key::A);
    chip8.insert_binding(0x6, Key::D);
    chip8.insert_binding(0x8, Key::S);
        
    let mut mem = Memory::from_args(env::args()).unwrap_or_else(|err| {
        eprintln!("Error while creating memory: {err}");
        process::exit(1);
    });
    
    if let Err(e) = chip8.run(&mut mem) {
        eprintln!("Error while running chip8: {e}");
        process::exit(1);
    }
}
 