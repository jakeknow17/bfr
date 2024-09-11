const INIT_TAPE_SIZE: usize = 65536;
const INIT_POINTER_LOC: usize = INIT_TAPE_SIZE / 2;

#[derive(Debug)]
enum Command {
    IncPointer,
    DecPointer,
    IncData,
    DecData,
    Output,
    Input,
    JumpForward { idx: usize },
    JumpBack { idx: usize },
}

fn parse(src: String) -> Vec<Command> {
    let mut commands: Vec<Command> = vec![];
    let mut stack: Vec<usize> = vec![];
    let mut idx = 0;
    for c in src.chars() {
        let op = match c {
            '>' => Some(Command::IncPointer),
            '<' => Some(Command::DecPointer),
            '+' => Some(Command::IncData),
            '-' => Some(Command::DecData),
            '.' => Some(Command::Output),
            ',' => Some(Command::Input),
            '[' => {
                stack.push(idx);
                // idx will be changed when the matching ']' is encountered
                Some(Command::JumpForward { idx: 0 })
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
                if let Command::JumpForward { idx: forward_idx } = &mut commands[prev_idx] {
                    *forward_idx = idx;
                } else {
                    eprintln!("Error parsing input: Unexpected character");
                    std::process::exit(1);
                }
                Some(Command::JumpBack { idx: prev_idx })
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

fn interp(commands: Vec<Command>) {
    let mut tape: Vec<u8> = vec![0; INIT_TAPE_SIZE];
    let mut pointer = INIT_POINTER_LOC;

    let mut pc = 0;

    while pc < commands.len() {
        match commands[pc] {
            Command::IncPointer => pointer += 1,
            Command::DecPointer => pointer -= 1,
            Command::IncData => tape[pointer] = tape[pointer].wrapping_add(1),
            Command::DecData => tape[pointer] = tape[pointer].wrapping_sub(1),
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
    interp(commands);
}
