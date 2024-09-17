mod parser;

const INIT_TAPE_SIZE: usize = 65536;
const INIT_POINTER_LOC: usize = INIT_TAPE_SIZE / 2;

struct Args {
    filename: String,
    enable_profiler: bool,
    enable_pretty_print: bool, // This disables interpretation
}

fn parse_args() -> Args {
    let mut tmp_filename = None;
    let mut args_struct = Args {
        filename: "".to_string(),
        enable_profiler: false,
        enable_pretty_print: false,
    };
    for arg in std::env::args().skip(1).peekable() {
        match arg.as_str() {
            "-p" => args_struct.enable_profiler = true,
            "--pretty" => args_struct.enable_pretty_print = true,
            _ => tmp_filename = Some(arg),
        }
    }

    // Requires filename
    match tmp_filename {
        Some(val) => args_struct.filename = val,
        None => {
            eprintln!("Usage: bfr <filename> [-p] [--pretty]");
            std::process::exit(1);
        }
    }
    
    args_struct
}


fn interp(commands: &mut [parser::Command], tape: &mut [u8], pointer: &mut usize, pc: &mut usize) {
    use parser::Command;

    while *pc < commands.len() {
        match &mut commands[*pc] {
            Command::IncPointer { ref mut count } => {
                *count += 1;
                *pointer += 1;
            },
            Command::DecPointer { ref mut count } => {
                *count += 1;
                *pointer -= 1;
            },
            Command::IncData { ref mut count } => {
                *count += 1;
                tape[*pointer] = tape[*pointer].wrapping_add(1);
            },
            Command::DecData { ref mut count } => {
                *count += 1;
                tape[*pointer] = tape[*pointer].wrapping_sub(1);
            },
            Command::Output { ref mut count } => {
                *count += 1;
                match char::from_u32(u32::from(tape[*pointer])) {
                    Some(c) => print!("{}", c),
                    None => {},
                }
            }
            Command::Input { ref mut count } => {
                use std::io::Read;

                *count += 1;
                let mut input_buf: [u8; 1] = [0; 1];
                std::io::stdin().read_exact(&mut input_buf).expect("Failed to read input");
                tape[*pointer] = input_buf[0];
            },
            Command::Loop { body, ref mut start_count, ref mut end_count } => {
                *start_count += 1;
                while tape[*pointer] != 0 {
                    let mut loop_pc = 0;
                    interp(body, tape, pointer, &mut loop_pc);

                    *end_count += 1;
                    if tape[*pointer] == 0 {
                        break;
                    }

                    *start_count += 1;
                }
            }
        }
        *pc += 1;
    }
}

fn main() {

    let args = parse_args();

    // Read the file contents
    let contents = match std::fs::read_to_string(&args.filename) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading file {}: {}", args.filename, e);
            std::process::exit(1);
        }
    };

    let mut commands = parser::parse(&contents);

    if args.enable_pretty_print {
        parser::pretty_print(&commands);
        return;
    }

    let mut tape: Vec<u8> = vec![0; INIT_TAPE_SIZE];
    let mut pointer = INIT_POINTER_LOC;
    let mut pc = 0;

    interp(&mut commands, &mut tape, &mut pointer, &mut pc);

    println!();
    println!("Terminated normally");

    if args.enable_profiler {
        parser::print_profile(&commands);
    }
}
