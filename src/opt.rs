use std::collections::HashMap;
use crate::parser::Command;

pub fn collapse(commands: Vec<Command>) -> Vec<Command> {
    let mut collapsed_commands: Vec<Command> = Vec::new();
    let mut index_map = HashMap::new(); // Old index to new index mapping
    let mut iter = commands.into_iter().peekable();
    let mut current_old_idx = 0;
    let mut current_new_idx = 0;

    // First, collapse consecutive commands
    while let Some(current_command) = iter.next() {
        match current_command {
            Command::IncPointer { amount } => {
                let mut total_amount = amount;
                while let Some(Command::IncPointer { amount: next_amount }) = iter.peek() {
                    total_amount += next_amount;
                    iter.next(); // Move past the collapsed command
                    current_old_idx += 1;
                }
                collapsed_commands.push(Command::IncPointer { amount: total_amount });
                index_map.insert(current_old_idx, current_new_idx);
                current_new_idx += 1;
            }
            Command::DecPointer { amount } => {
                let mut total_amount = amount;
                while let Some(Command::DecPointer { amount: next_amount }) = iter.peek() {
                    total_amount += next_amount;
                    iter.next(); // Move past the collapsed command
                    current_old_idx += 1;
                }
                collapsed_commands.push(Command::DecPointer { amount: total_amount });
                index_map.insert(current_old_idx, current_new_idx);
                current_new_idx += 1;
            }
            Command::IncData { offset, amount } => {
                let mut total_amount = amount;
                while let Some(Command::IncData { offset: next_offset, amount: next_amount }) = iter.peek() {
                    if *next_offset == offset {
                        total_amount += next_amount;
                        iter.next(); // Move past the collapsed command
                        current_old_idx += 1;
                    } else {
                        break; // Offset is different, stop collapsing
                    }
                }
                collapsed_commands.push(Command::IncData { offset, amount: total_amount });
                index_map.insert(current_old_idx, current_new_idx);
                current_new_idx += 1;
            }
            Command::DecData { offset, amount } => {
                let mut total_amount = amount;
                while let Some(Command::DecData { offset: next_offset, amount: next_amount }) = iter.peek() {
                    if *next_offset == offset {
                        total_amount += next_amount;
                        iter.next(); // Move past the collapsed command
                        current_old_idx += 1;
                    } else {
                        break; // Offset is different, stop collapsing
                    }
                }
                collapsed_commands.push(Command::DecData { offset, amount: total_amount });
                index_map.insert(current_old_idx, current_new_idx);
                current_new_idx += 1;
            }
            // Non-collapsible commands
            Command::Output | Command::Input | Command::ZeroData { .. } => {
                collapsed_commands.push(current_command);
                index_map.insert(current_old_idx, current_new_idx);
                current_new_idx += 1;
            }
            // Store jump commands but fix them later
            Command::JumpForward { .. } | Command::JumpBack { .. } => {
                collapsed_commands.push(current_command);
                index_map.insert(current_old_idx, current_new_idx);
                current_new_idx += 1;
            }
        }
        current_old_idx += 1;
    }

    // Second pass to adjust jump targets
    for command in &mut collapsed_commands {
        match command {
            Command::JumpForward { idx } => {
                // Update the jump target using the index map
                *idx = *index_map.get(idx).unwrap();
            }
            Command::JumpBack { idx } => {
                // Update the jump target using the index map
                *idx = *index_map.get(idx).unwrap();
            }
            _ => {}
        }
    }

    collapsed_commands
}


pub fn optimize(commands: Vec<Command>) -> Vec<Command> {
    let commands = collapse(commands);
    commands
}
