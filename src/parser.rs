#[derive(Debug)]
pub enum Command {
    IncPointer { amount: usize },
    DecPointer { amount: usize },
    IncData { offset: isize, amount: u8 },
    DecData { offset: isize, amount: u8 },
    ZeroData { offset: isize },
    Output,
    Input,
    JumpForward { idx: usize },
    JumpBack { idx: usize },
}

pub fn parse(src: String) -> Vec<Command> {
    let mut commands: Vec<Command> = vec![];
    let mut stack: Vec<usize> = vec![];
    let mut idx = 0;
    for c in src.chars() {
        let cmd = match c {
            '>' => Some(Command::IncPointer { amount: 1 }),
            '<' => Some(Command::DecPointer { amount: 1 }),
            '+' => Some(Command::IncData { offset: 0, amount: 1 }),
            '-' => Some(Command::DecData { offset: 0, amount: 1 }),
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

        if let Some(cmd) = cmd {
            commands.push(cmd);
            idx += 1;
        }
    }

    if stack.len() > 0 {
        eprintln!("Error parsing input: Unmatched '['");
        std::process::exit(1);
    }

    commands
}
