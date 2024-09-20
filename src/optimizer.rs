use crate::parser::Command;

pub fn collapse(commands: &mut Vec<Command>) {
    let mut read_idx = 0;
    let mut write_idx = 0;

    while read_idx < commands.len() {
        let current_command = &commands[read_idx];

        match current_command {
            Command::IncPointer { amount, .. } => {
                let mut total_amount = *amount;
                while read_idx + 1 < commands.len() {
                    if let Command::IncPointer { amount: next_amount, .. } = &commands[read_idx + 1] {
                        total_amount += next_amount;
                        read_idx += 1;
                    } else {
                        break;
                    }
                }
                commands[write_idx] = Command::IncPointer { amount: total_amount, count: 0 };
            }
            Command::DecPointer { amount, .. } => {
                let mut total_amount = *amount;
                while read_idx + 1 < commands.len() {
                    if let Command::DecPointer { amount: next_amount, .. } = &commands[read_idx + 1] {
                        total_amount += next_amount;
                        read_idx += 1;
                    } else {
                        break;
                    }
                }
                commands[write_idx] = Command::DecPointer { amount: total_amount, count: 0 };
            }
            Command::IncData { offset, amount, .. } => {
                let mut total_amount = *amount;
                while read_idx + 1 < commands.len() {
                    if let Command::IncData { offset: next_offset, amount: next_amount, .. } = &commands[read_idx + 1] {
                        if next_offset == offset {
                            total_amount += next_amount;
                            read_idx += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                commands[write_idx] = Command::IncData { offset: *offset, amount: total_amount, count: 0 };
            }
            Command::DecData { offset, amount, .. } => {
                let mut total_amount = *amount;
                while read_idx + 1 < commands.len() {
                    if let Command::DecData { offset: next_offset, amount: next_amount, .. } = &commands[read_idx + 1] {
                        if next_offset == offset {
                            total_amount += next_amount;
                            read_idx += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                commands[write_idx] = Command::DecData { offset: *offset, amount: total_amount, count: 0 };
            }
            // Non-collapsible commands
            Command::SetData { .. } | Command::Output { .. } | Command::Input { .. } => {
                commands[write_idx] = current_command.clone();
            }
            Command::Loop { .. } => {
                // Replace with empty loop at the same spot in array instead of cloning
                let mut current_loop = std::mem::replace(&mut commands[read_idx], Command::Loop { body: vec![], id: 0, start_count: 0, end_count: 0 });
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


pub fn optimize(commands: &mut Vec<Command>, optimization_level: u8) {
    if optimization_level > 0 {
        collapse(commands);
    }
}
