pub mod parser;
pub mod opt;

use parser::parse;
use parser::Command;
use opt::optimize;

const INIT_TAPE_SIZE: usize = 65536;
const INIT_POINTER_LOC: usize = INIT_TAPE_SIZE / 2;

fn interp(commands: Vec<Command>) {
    let mut tape: Vec<u8> = vec![0; INIT_TAPE_SIZE];
    let mut pointer = INIT_POINTER_LOC;

    let mut pc = 0;

    while pc < commands.len() {
        match commands[pc] {
            Command::IncPointer { amount } => pointer += amount,
            Command::DecPointer { amount } => pointer -= amount,
            Command::IncData { offset, amount } => tape[pointer.saturating_add_signed(offset)] = tape[pointer].wrapping_add(amount),
            Command::DecData { offset, amount } => tape[pointer.saturating_add_signed(offset)] = tape[pointer].wrapping_sub(amount),
            Command::ZeroData { offset } => tape[pointer.saturating_add_signed(offset)] = 0,
            Command::Output => {
                match char::from_u32(u32::from(tape[pointer])) {
                    Some(c) => print!("{}", c),
                    None => {},
                }
            }
            Command::Input => {
                use std::io::Read;

                let mut input_buf: [u8; 1] = [0; 1];
                std::io::stdin().read_exact(&mut input_buf).expect("Failed to read input");
                tape[pointer] = input_buf[0];
            },
            Command::JumpForward { idx} => if tape[pointer] == 0 { pc = idx },
            Command::JumpBack { idx } => if tape[pointer] != 0 { pc = idx },
        }
        pc += 1;
    }

    println!("\nExited normally\n");
}

fn main() {
    // Get args
    let args: Vec<String> = std::env::args().collect();

    // Require at least 2 args
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

     // Get the filename from args[1]
    let filename = &args[1];

    // Read the file contents
    let contents = match std::fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    let commands = parse(contents);
    let opt_commands = optimize(commands);
    interp(opt_commands);
}
