const INIT_TAPE_SIZE: usize = 8192;

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

fn parse(src: String) {
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
                Some(Command::JumpForward { idx: 0 })
            },
            ']' => {
                let prevIdx = match stack.pop() {
                    Some(value) => value,
                    None => {
                        eprintln!("Error parsing input: Unmatched ']'");
                        std::process::exit(1);
                    }
                };
                if let Command::JumpForward { idx: forwardIdx } = &mut commands[prevIdx] {
                    *forwardIdx = idx;
                } else {
                    eprintln!("Error parsing input: Unexpected character");
                    std::process::exit(1);
                }
                Some(Command::JumpBack { idx: prevIdx })
            },
            _ => None,
        };

        idx += 1;
    }
}

//fn interp(src: &str) {
//    let mut tape: Vec<u8> = vec![0; INIT_TAPE_SIZE];
//    let mut pointer = 0;
//}

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

    parse(contents);
}
