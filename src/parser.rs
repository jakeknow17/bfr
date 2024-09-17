#[derive(Debug)]
pub enum Command {
    IncPointer { count: usize },
    DecPointer { count: usize },
    IncData { count: usize },
    DecData { count: usize },
    Output { count: usize },
    Input { count: usize },
    Loop { body: Vec<Command>, start_count: usize, end_count: usize },
}

pub fn parse(src: &String) -> Vec<Command> {
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

pub fn pretty_print(commands: &[Command]) {
    fn pretty_print_rec(commands: &[Command], indent_level: usize, newline_end: &mut bool) {
        let indent = "  ".repeat(indent_level);

        for command in commands {
            if *newline_end {
                print!("{}", indent);
            }
            match command {
                Command::IncPointer { .. } => { 
                    print!(">"); 
                    *newline_end = false;
                },
                Command::DecPointer { .. } => { 
                    print!("<"); 
                    *newline_end = false;
                },
                Command::IncData { .. } => { 
                    print!("+"); 
                    *newline_end = false;
                },
                Command::DecData { .. } => { 
                    print!("-"); 
                    *newline_end = false;
                },
                Command::Output { .. } => { 
                    print!("."); 
                    *newline_end = false;
                },
                Command::Input { .. } => { 
                    print!(","); 
                    *newline_end = false;
                },
                Command::Loop { body, .. } => {
                    if !*newline_end {
                        println!();
                        print!("{}", indent);
                    }
                    println!("[");
                    *newline_end = true;

                    // Recursively pretty print the commands inside the loop
                    pretty_print_rec(body, indent_level + 1, newline_end);

                    if !*newline_end { println!() }
                    println!("{}]", indent);
                    *newline_end = true;
                },
            }
        }

        if !*newline_end {
            println!();
            *newline_end = true;
        }
    }

    // Driver for recursive method
    let mut newline_end = true;
    pretty_print_rec(&commands, 0, &mut newline_end);
}
