use chip8::{Chip8, Memory};
use std::collections::HashMap;
use minifb::Key;
use std::process;
use std::env;

fn main() {
    let mut bindings = HashMap::new();
        bindings.insert(0x0, Key::Key0);
        bindings.insert(0x1, Key::Key1);
        bindings.insert(0x2, Key::W);
        bindings.insert(0x3, Key::Key3);
        bindings.insert(0x4, Key::A);
        bindings.insert(0x5, Key::Key5);
        bindings.insert(0x6, Key::D);
        bindings.insert(0x7, Key::Key7);
        bindings.insert(0x8, Key::S);
        bindings.insert(0x9, Key::Key9);
        bindings.insert(0xa, Key::X);
        bindings.insert(0xb, Key::B);
        bindings.insert(0xc, Key::C);
        bindings.insert(0xd, Key::D);
        bindings.insert(0xe, Key::E);
        bindings.insert(0xf, Key::F);

    let mut chip8 = Chip8::with(bindings);
    chip8.set_colors(0x800080, 0xffc0cb); // purple and pink
        
    let mut mem = Memory::from_args(env::args()).unwrap_or_else(|err| {
        eprintln!("Error while creating memory: {err}");
        process::exit(1);
    });
    
    if let Err(e) = chip8.run(&mut mem) {
        eprintln!("Error while running chip8: {e}");
        process::exit(1);
    }
}
 