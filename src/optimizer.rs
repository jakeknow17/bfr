use crate::parser::{Command, Direction};

pub fn is_simple_loop(loop_cmd: &Command) -> (bool, isize) {
    if let Command::Loop { body, .. } = loop_cmd {
        let mut loop_ptr: isize = 0;
        let mut induction_delta: isize = 0;

        for cmd in body {
            match cmd {
                Command::IncPointer { amount, .. } => loop_ptr += *amount as isize,
                Command::DecPointer { amount, .. } => loop_ptr -= *amount as isize,
                Command::IncData { offset, amount, .. } => {
                    if loop_ptr.wrapping_add(*offset) == 0 {
                        induction_delta += *amount as isize
                    }
                }
                Command::DecData { offset, amount, .. } => {
                    if loop_ptr.wrapping_add(*offset) == 0 {
                        induction_delta -= *amount as isize
                    }
                }
                _ => return (false, 0),
            }
        }
        if loop_ptr == 0 && (induction_delta == -1 || induction_delta == 1) {
            return (true, induction_delta);
        } else {
            return (false, 0);
        }
    } else {
        return (false, 0);
    }
}

pub fn collapse(commands: &mut Vec<Command>) {
    let mut read_idx = 0;
    let mut write_idx = 0;

    while read_idx < commands.len() {
        let current_command = &commands[read_idx];

        match current_command {
            Command::IncPointer { amount, .. } => {
                let mut total_amount: isize = *amount as isize;
                while read_idx + 1 < commands.len() {
                    match &commands[read_idx + 1] {
                        Command::IncPointer {
                            amount: next_amount,
                            ..
                        } => {
                            total_amount += *next_amount as isize;
                            read_idx += 1;
                        }
                        Command::DecPointer {
                            amount: next_amount,
                            ..
                        } => {
                            total_amount -= *next_amount as isize;
                            read_idx += 1;
                        }
                        _ => break,
                    }
                }

                if total_amount > 0 {
                    commands[write_idx] = Command::IncPointer {
                        amount: total_amount as usize,
                        count: 0,
                    };
                } else if total_amount < 0 {
                    commands[write_idx] = Command::DecPointer {
                        amount: (-total_amount) as usize,
                        count: 0,
                    };
                } else {
                    read_idx += 1; // Don't increment write idx
                    continue;
                }
            }
            Command::DecPointer { amount, .. } => {
                let mut total_amount: isize = *amount as isize;
                while read_idx + 1 < commands.len() {
                    match &commands[read_idx + 1] {
                        Command::DecPointer {
                            amount: next_amount,
                            ..
                        } => {
                            total_amount += *next_amount as isize;
                            read_idx += 1;
                        }
                        Command::IncPointer {
                            amount: next_amount,
                            ..
                        } => {
                            total_amount -= *next_amount as isize;
                            read_idx += 1;
                        }
                        _ => break,
                    }
                }

                if total_amount > 0 {
                    commands[write_idx] = Command::DecPointer {
                        amount: total_amount as usize,
                        count: 0,
                    };
                } else if total_amount < 0 {
                    commands[write_idx] = Command::IncPointer {
                        amount: (-total_amount) as usize,
                        count: 0,
                    };
                } else {
                    read_idx += 1; // Don't increment write idx
                    continue;
                }
            }
            Command::IncData { offset, amount, .. } => {
                let mut total_amount: isize = *amount as isize;
                while read_idx + 1 < commands.len() {
                    match &commands[read_idx + 1] {
                        Command::IncData {
                            offset: next_offset,
                            amount: next_amount,
                            ..
                        } => {
                            if next_offset == offset {
                                total_amount += *next_amount as isize;
                                read_idx += 1;
                            } else {
                                break;
                            }
                        }
                        Command::DecData {
                            offset: next_offset,
                            amount: next_amount,
                            ..
                        } => {
                            if next_offset == offset {
                                total_amount -= *next_amount as isize;
                                read_idx += 1;
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }

                if total_amount > 0 {
                    commands[write_idx] = Command::IncData {
                        offset: *offset,
                        amount: total_amount as u8,
                        count: 0,
                    };
                } else if total_amount < 0 {
                    commands[write_idx] = Command::DecData {
                        offset: *offset,
                        amount: (-total_amount) as u8,
                        count: 0,
                    };
                } else {
                    read_idx += 1; // Don't increment write idx
                    continue;
                }
            }
            Command::DecData { offset, amount, .. } => {
                let mut total_amount: isize = *amount as isize;
                while read_idx + 1 < commands.len() {
                    match &commands[read_idx + 1] {
                        Command::DecData {
                            offset: next_offset,
                            amount: next_amount,
                            ..
                        } => {
                            if next_offset == offset {
                                total_amount += *next_amount as isize;
                                read_idx += 1;
                            } else {
                                break;
                            }
                        }
                        Command::IncData {
                            offset: next_offset,
                            amount: next_amount,
                            ..
                        } => {
                            if next_offset == offset {
                                total_amount -= *next_amount as isize;
                                read_idx += 1;
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }

                if total_amount > 0 {
                    commands[write_idx] = Command::DecData {
                        offset: *offset,
                        amount: total_amount as u8,
                        count: 0,
                    };
                } else if total_amount < 0 {
                    commands[write_idx] = Command::IncData {
                        offset: *offset,
                        amount: (-total_amount) as u8,
                        count: 0,
                    };
                } else {
                    read_idx += 1; // Don't increment write idx
                    continue;
                }
            }
            // Non-collapsible commands
            Command::SetData { .. }
            | Command::Scan { .. }
            | Command::Output { .. }
            | Command::Input { .. }
            | Command::AddOffsetData { .. }
            | Command::SubOffsetData { .. } => {
                commands[write_idx] = current_command.clone();
            }
            Command::Loop { .. } => {
                // Replace with empty loop at the same spot in array instead of cloning
                let mut current_loop = std::mem::replace(
                    &mut commands[read_idx],
                    Command::Loop {
                        body: vec![],
                        id: 0,
                        start_count: 0,
                        end_count: 0,
                    },
                );
                if let Command::Loop { ref mut body, .. } = current_loop {
                    collapse(body);
                }
                commands[write_idx] = current_loop;
            }
        }
        read_idx += 1;
        write_idx += 1;
    }

    commands.truncate(write_idx);
}

fn fold_zero_loop(commands: &mut Vec<Command>) {
    for i in 0..commands.len() {
        let current_command = &mut commands[i];

        match current_command {
            Command::Loop { ref mut body, .. } => {
                if body.len() != 1 {
                    fold_zero_loop(body);
                    continue;
                }
                if let Command::DecData { offset, amount, .. } = &body[0] {
                    // Even amount can cause infinite loop
                    if *offset != 0 || *amount % 2 == 0 {
                        continue;
                    }
                    commands[i] = Command::SetData {
                        offset: 0,
                        value: 0,
                        count: 0,
                    };
                } else if let Command::IncData { offset, amount, .. } = &body[0] {
                    if *offset != 0 || *amount % 2 == 0 {
                        continue;
                    }
                    commands[i] = Command::SetData {
                        offset: 0,
                        value: 0,
                        count: 0,
                    };
                } else {
                    fold_zero_loop(body);
                }
            }
            _ => (),
        }
    }
}

fn replace_simple_loops(commands: &mut Vec<Command>) {
    let mut i = 0;
    while i < commands.len() {
        let current_command = &mut commands[i];

        let (is_simple, induction_delta) = is_simple_loop(current_command);
        match current_command {
            Command::Loop { ref mut body, .. } => {
                if !is_simple {
                    i += 1;
                    replace_simple_loops(body);
                    continue;
                }
                let mut new_cmds: Vec<Command> = vec![];
                let mut loop_ptr: isize = 0;
                for cmd in body {
                    match cmd {
                        Command::IncPointer { amount, .. } => loop_ptr += *amount as isize,
                        Command::DecPointer { amount, .. } => loop_ptr -= *amount as isize,
                        Command::IncData { offset, amount, .. } => {
                            if loop_ptr.wrapping_add(*offset) != 0 {
                                let new_cmd = Command::AddOffsetData {
                                    dest_offset: loop_ptr + *offset,
                                    src_offset: 0,
                                    multiplier: *amount as usize,
                                    inverted: induction_delta == 1,
                                    count: 0,
                                };
                                new_cmds.push(new_cmd);
                            }
                        }
                        Command::DecData { offset, amount, .. } => {
                            if loop_ptr.wrapping_add(*offset) != 0 {
                                let new_cmd = Command::SubOffsetData {
                                    dest_offset: loop_ptr + *offset,
                                    src_offset: 0,
                                    multiplier: *amount as usize,
                                    inverted: induction_delta == 1,
                                    count: 0,
                                };
                                new_cmds.push(new_cmd);
                            }
                        }
                        _ => (),
                    }
                }
                new_cmds.push(Command::SetData {
                    offset: 0,
                    value: 0,
                    count: 0,
                });
                commands.splice(i..i + 1, new_cmds);
            }
            _ => (),
        }
        i += 1;
    }
}

fn replace_scans(commands: &mut Vec<Command>) {
    for i in 0..commands.len() {
        let current_command = &mut commands[i];

        match current_command {
            Command::Loop {
                id, ref mut body, ..
            } => {
                if body.len() != 1 {
                    fold_zero_loop(body);
                    continue;
                }
                if let Command::IncPointer { amount, .. } = &body[0] {
                    commands[i] = Command::Scan {
                        id: *id,
                        direction: Direction::Right,
                        skip_amount: *amount,
                        count: 0,
                    }
                } else if let Command::DecPointer { amount, .. } = &body[0] {
                    commands[i] = Command::Scan {
                        id: *id,
                        direction: Direction::Left,
                        skip_amount: *amount,
                        count: 0,
                    }
                } else {
                    replace_scans(body);
                }
            }
            _ => (),
        }
    }
}

pub fn optimize(commands: &mut Vec<Command>, optimization_level: u8) {
    if optimization_level > 0 {
        collapse(commands);
    }
    if optimization_level > 1 {
        fold_zero_loop(commands);
    }
    if optimization_level > 2 {
        replace_simple_loops(commands);
        replace_scans(commands);
    }
}
