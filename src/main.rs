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
    Loop { body: Vec<Command>, start_count: usize, end_count: usize },
}

fn parse(src: String) -> Vec<Command> {
    let mut commands: Vec<Command> = vec![];
    let mut stack: Vec<Vec<Command>> = vec![];

    for c in src.chars() {
        let op = match c {
            '>' => Some(Command::IncPointer { count: 0}),
            '<' => Some(Command::DecPointer { count: 0}),
            '+' => Some(Command::IncData { count: 0}),
            '-' => Some(Command::DecData { count: 0}),
            '.' => Some(Command::Output { count: 0}),
            ',' => Some(Command::Input { count: 0}),
            '[' => {
                stack.push(vec![]);
                None
            },
            ']' => {
                let loop_commands = match stack.pop() {
                    Some(cmds) => cmds,
                    None => {
                        eprintln!("Error parsing input: Unmatched ']'");
                        std::process::exit(1);
                    }
                };
                // Wrap loop commands in a Loop variant and push it to the current scope
                if let Some(inner_commands) = stack.last_mut() {
                    inner_commands.push(Command::Loop { body: loop_commands, start_count: 0, end_count: 0 });
                } else {
                    commands.push(Command::Loop { body: loop_commands, start_count: 0, end_count: 0 });
                }
                None
            },
            _ => None,
        };

        if let Some(op) = op {
            if let Some(inner_commands) = stack.last_mut() {
                inner_commands.push(op);
            } else {
                commands.push(op);
            }
        }
    }

    if stack.len() > 0 {
        eprintln!("Error parsing input: Unmatched '['");
        std::process::exit(1);
    }

    commands
}

fn interp(commands: &mut Vec<Command>, tape: &mut Vec<u8>, pointer: &mut usize, pc: &mut usize) {

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

    let mut commands = parse(contents);

    let mut tape: Vec<u8> = vec![0; INIT_TAPE_SIZE];
    let mut pointer = INIT_POINTER_LOC;
    let mut pc = 0;

    interp(&mut commands, &mut tape, &mut pointer, &mut pc);
    println!("\nTerminated normally");
}
