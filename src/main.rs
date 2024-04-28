mod chip8;

fn main() {
    let mut chip8 = chip8::Chip8::new();
    let mut mem = match chip8::memory::Memory::from(r"C:\Users\Erykoo\Documents\rust_projects\chip8\src\mem.txt") {
        Ok(mem) => mem,
        Err(e) => panic!("Error creating memory {}", e)
    };
    chip8.run(&mut mem);
}
