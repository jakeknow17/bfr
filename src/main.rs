const INIT_TAPE_SIZE: usize = 65536;
const INIT_POINTER_LOC: usize = INIT_TAPE_SIZE / 2;

struct Args {
    filename: String,
    enable_profiler: bool,
}

fn parse_args() -> Args {
    let mut tmp_filename = None;
    let mut args_struct = Args {
        filename: "".to_string(),
        enable_profiler: false,
    };
    for arg in std::env::args().skip(1) {
        if arg == "-p" {
            args_struct.enable_profiler = true;
        } else {
            tmp_filename = Some(arg);
        }
    }

    // Requires filename
    match tmp_filename {
        Some(val) => args_struct.filename = val,
        None => {
            eprintln!("Usage: bfr <filename> [-p]");
            std::process::exit(1);
        }
    }
    
    args_struct
}

#[derive(Debug)]
enum Command {
    IncPointer { count: usize },
    DecPointer { count: usize },
    IncData { count: usize },
    DecData { count: usize },
    Output { count: usize },
    Input { count: usize },
    JumpForward { idx: usize, count: usize },
    JumpBack { idx: usize, count: usize },
}

fn parse(src: String) -> Vec<Command> {
    let mut commands: Vec<Command> = vec![];
    let mut stack: Vec<usize> = vec![];
    let mut idx = 0;
    for c in src.chars() {
        let op = match c {
            '>' => Some(Command::IncPointer { count: 0}),
            '<' => Some(Command::DecPointer { count: 0}),
            '+' => Some(Command::IncData { count: 0}),
            '-' => Some(Command::DecData { count: 0}),
            '.' => Some(Command::Output { count: 0}),
            ',' => Some(Command::Input { count: 0}),
            '[' => {
                stack.push(idx);
                // idx will be changed when the matching ']' is encountered
                Some(Command::JumpForward { idx: 0, count: 0 })
            },
            ']' => {
                let prev_idx = match stack.pop() {
                    Some(value) => value,
                    None => {
                        eprintln!("Error parsing input: Unmatched ']'");
                        std::process::exit(1);
                    }
                };
                // The command at prev_idx should always be JumpForward
                if let Command::JumpForward { idx: ref mut forward_idx, .. } = &mut commands[prev_idx] {
                    *forward_idx = idx;
                } else {
                    eprintln!("Error parsing input: Unexpected character");
                    std::process::exit(1);
                }
                Some(Command::JumpBack { idx: prev_idx, count: 0 })
            },
            _ => None,
        };

        if let Some(op) = op {
            commands.push(op);
            idx += 1;
        }
    }

    if stack.len() > 0 {
        eprintln!("Error parsing input: Unmatched '['");
        std::process::exit(1);
    }

    commands
}

fn interp(mut commands: Vec<Command>) {
    let mut tape: Vec<u8> = vec![0; INIT_TAPE_SIZE];
    let mut pointer = INIT_POINTER_LOC;

    let mut pc = 0;

    while pc < commands.len() {
        match commands[pc] {
            Command::IncPointer { ref mut count } => {
                *count += 1;
                pointer += 1;
            },
            Command::DecPointer { ref mut count } => {
                *count += 1;
                pointer -= 1;
            },
            Command::IncData { ref mut count } => {
                *count += 1;
                tape[pointer] = tape[pointer].wrapping_add(1);
            },
            Command::DecData { ref mut count } => {
                *count += 1;
                tape[pointer] = tape[pointer].wrapping_sub(1);
            },
            Command::Output { ref mut count } => {
                *count += 1;
                match char::from_u32(u32::from(tape[pointer])) {
                    Some(c) => print!("{}", c),
                    None => {},
                }
            }
            Command::Input { ref mut count } => {
                use std::io::Read;

                *count += 1;
                let mut input_buf: [u8; 1] = [0; 1];
                std::io::stdin().read_exact(&mut input_buf).expect("Failed to read input");
                tape[pointer] = input_buf[0];
            },
            Command::JumpForward { idx, ref mut count } => {
                *count += 1;
                if tape[pointer] == 0 { pc = idx }
            },
            Command::JumpBack { idx, ref mut count } => {
                *count += 1;
                if tape[pointer] != 0 { pc = idx }
            },
        }
        pc += 1;
    }

    println!("\nExited normally\n");
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

    let commands = parse(contents);
    interp(commands);
}
