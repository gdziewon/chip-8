mod chip8;

fn main() {
    
    let mut chip8 = chip8::Chip8::new();
    let mut mem = chip8::memory::Memory::new();
    
    // Draw commands refactored with updated memory values
    mem.set_mem(0x200, 0x63); // V3 = 5
    mem.set_mem(0x201, 0x05);

    mem.set_mem(0x202, 0xd1); // draw
    mem.set_mem(0x203, 0x05);

    mem.set_mem(0x204, 0x81); // add v1 += v3
    mem.set_mem(0x205, 0x34);

    mem.set_mem(0x206, 0xb2); // jmp 0x202
    mem.set_mem(0x207, 0x02);

    chip8.run(&mut mem);
}
