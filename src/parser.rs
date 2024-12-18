#[derive(Debug, Clone)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum Command {
    /// Repr: `>{amount if amount > 1}`
    IncPointer {
        amount: usize,
        count: usize,
    },
    /// Repr: `<{amount if amount > 1}`
    DecPointer {
        amount: usize,
        count: usize,
    },
    /// Repr: `+|offset if offset != 0|{amount if amount > 1}`
    IncData {
        offset: isize,
        amount: u8,
        count: usize,
    },
    /// Repr: `-|offset if offset != 0|{amount if amount > 1}`
    DecData {
        offset: isize,
        amount: u8,
        count: usize,
    },
    /// Repr: `=|offset if offset != 0|{amount}`
    SetData {
        offset: isize,
        value: u8,
        count: usize,
    },
    /// Repr: `S>{skip_amount if skip_amount > 1}` if direction is right
    /// Repr: `S<{skip_amount if skip_amount > 1}` if direction is left
    Scan {
        id: usize,
        direction: Direction,
        skip_amount: usize,
        count: usize,
    },
    /// Repr: `a+|dest_offset||src_offset|{multiplier}` if inverted
    /// Repr: `a-|dest_offset||src_offset|{multiplier}` if not inverted
    AddOffsetData {
        dest_offset: isize,
        src_offset: isize,
        multiplier: usize,
        inverted: bool,
        count: usize,
    },
    /// Repr: `s+|dest_offset||src_offset|{multiplier}` if inverted
    /// Repr: `s-|dest_offset||src_offset|{multiplier}` if not inverted
    SubOffsetData {
        dest_offset: isize,
        src_offset: isize,
        multiplier: usize,
        inverted: bool,
        count: usize,
    },
    /// Repr: `.{value}` if out_type is const
    /// Repr: `.|offset if offset != 0|` if out_type is const
    Output {
        out_type: OutputType,
        count: usize,
    },
    /// Repr: `,|offset if offset != 0|`
    Input {
        offset: isize,
        count: usize,
    },
    /// Repr: `[ body ]`
    Loop {
        id: usize,
        body: Vec<Command>,
        start_count: usize,
        end_count: usize,
    },
}

#[derive(Debug, Clone)]
pub enum OutputType {
    Const(u8),
    Cell { offset: isize },
}

pub fn parse(src: &String) -> Vec<Command> {
    let mut commands: Vec<Command> = vec![];
    let mut stack: Vec<Vec<Command>> = vec![];
    let mut loop_id: usize = 0;

    for c in src.chars() {
        let op = match c {
            '>' => Some(Command::IncPointer {
                amount: 1,
                count: 0,
            }),
            '<' => Some(Command::DecPointer {
                amount: 1,
                count: 0,
            }),
            '+' => Some(Command::IncData {
                offset: 0,
                amount: 1,
                count: 0,
            }),
            '-' => Some(Command::DecData {
                offset: 0,
                amount: 1,
                count: 0,
            }),
            '.' => Some(Command::Output {
                out_type: OutputType::Cell { offset: 0 },
                count: 0,
            }),
            ',' => Some(Command::Input {
                offset: 0,
                count: 0,
            }),
            '[' => {
                stack.push(vec![]);
                None
            }
            ']' => {
                loop_id += 1;

                let loop_commands = match stack.pop() {
                    Some(cmds) => cmds,
                    None => {
                        eprintln!("Error parsing input: Unmatched ']'");
                        std::process::exit(1);
                    }
                };
                // Wrap loop commands in a Loop variant and push it to the current scope
                if let Some(inner_commands) = stack.last_mut() {
                    inner_commands.push(Command::Loop {
                        body: loop_commands,
                        id: loop_id,
                        start_count: 0,
                        end_count: 0,
                    });
                } else {
                    commands.push(Command::Loop {
                        body: loop_commands,
                        id: loop_id,
                        start_count: 0,
                        end_count: 0,
                    });
                }
                None
            }
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
                Command::IncPointer { amount, .. } => {
                    if *amount == 1 {
                        print!(">");
                    } else {
                        print!("(>{})", amount);
                    }
                    *newline_end = false;
                }
                Command::DecPointer { amount, .. } => {
                    if *amount == 1 {
                        print!("<");
                    } else {
                        print!("(<{})", amount);
                    }
                    *newline_end = false;
                }
                Command::IncData { offset, amount, .. } => {
                    let offset_str = if *offset == 0 {
                        String::from("")
                    } else {
                        format!("({})", offset)
                    };
                    if *amount == 1 {
                        print!("{}+", offset_str);
                    } else {
                        print!("({}+{})", offset_str, amount);
                    }
                    *newline_end = false;
                }
                Command::DecData { offset, amount, .. } => {
                    let offset_str = if *offset == 0 {
                        String::from("")
                    } else {
                        format!("({})", offset)
                    };
                    if *amount == 1 {
                        print!("{}-", offset_str);
                    } else {
                        print!("({}-{})", offset_str, amount);
                    }
                    *newline_end = false;
                }
                Command::SetData { offset, value, .. } => {
                    let offset_str = if *offset == 0 {
                        String::from("")
                    } else {
                        format!("({})", offset)
                    };
                    print!("({}={})", offset_str, value);
                    *newline_end = false;
                }
                Command::Scan {
                    direction,
                    skip_amount,
                    ..
                } => {
                    match direction {
                        Direction::Left => print!("[(<{})]", skip_amount),
                        Direction::Right => print!("[(>{})]", skip_amount),
                    }
                    *newline_end = false;
                }
                Command::AddOffsetData {
                    dest_offset,
                    src_offset,
                    multiplier,
                    inverted,
                    ..
                } => {
                    let mut dest_string = String::new();
                    let inverted_str = if *inverted { "-" } else { "" };
                    dest_string
                        .push_str(&format!("{}({}*{})", inverted_str, src_offset, multiplier));
                    print!("({}+={})", dest_offset, dest_string);
                    *newline_end = false;
                }
                Command::SubOffsetData {
                    dest_offset,
                    src_offset,
                    multiplier,
                    inverted,
                    ..
                } => {
                    let mut dest_string = String::new();
                    let inverted_str = if *inverted { "-" } else { "" };
                    dest_string
                        .push_str(&format!("{}({}*{})", inverted_str, src_offset, multiplier));
                    print!("({}-={})", dest_offset, dest_string);
                    *newline_end = false;
                }
                Command::Output { out_type, .. } => {
                    match out_type {
                        OutputType::Const(val) => {
                            print!("(.{val})");
                        }
                        OutputType::Cell { offset: _ } => {
                            print!(".");
                        }
                    }
                    *newline_end = false;
                }
                Command::Input { .. } => {
                    print!(",");
                    *newline_end = false;
                }
                Command::Loop { body, .. } => {
                    if !*newline_end {
                        println!();
                        print!("{}", indent);
                    }
                    println!("[");
                    *newline_end = true;

                    // Recursively pretty print the commands inside the loop
                    pretty_print_rec(body, indent_level + 1, newline_end);

                    if !*newline_end {
                        println!()
                    }
                    println!("{}]", indent);
                    *newline_end = true;
                }
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
