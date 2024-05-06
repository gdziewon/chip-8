use chip8::{Chip8, Memory};
use std::process;
use std::env;

fn main() {
    let mut chip8 = Chip8::new();
        
    let mut mem = Memory::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Error while creating memory: {err}");
        process::exit(1);
    });
    
    if let Err(e) = chip8.run(&mut mem) {
        eprintln!("Error while running chip8: {e}");
        process::exit(1);
    }
}
 