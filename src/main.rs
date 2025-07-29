use chip8::Chip8;
use std::fs::File;
use std::process;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf
}

fn main() {
    let args = Args::parse();
    let program = &File::open(args.path).unwrap();

    let mut chip8 = Chip8::new();
    chip8.load_program(program).unwrap();
    chip8.set_colors(0x800080, 0xffc0cb); // purple and pink

    if let Err(e) = chip8.run() {
        eprintln!("Error while running chip8: {e}");
        process::exit(1);
    }
}
