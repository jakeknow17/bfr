use crate::parser::{pretty_print, Command, Direction, OutputType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum AbstractCell {
    Value(u8),
    Top,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(unused)]
enum AbstractPointer {
    Value(usize),
    Top,
}

struct IOCommand {
    command: Command,
    pointer: usize,
}

const INIT_POINTER_LOC: usize = 0x4000;

fn check_loop_pointer(command: &Command) -> bool {
    fn check_loop_pointer_rec(command: &Command, rel_pointer: &mut isize) -> bool {
        match command {
            Command::IncPointer { amount, .. } => {
                *rel_pointer += *amount as isize;
                true
            }
            Command::DecPointer { amount, .. } => {
                *rel_pointer -= *amount as isize;
                true
            }
            Command::Scan {
                id: _,
                direction: _,
                skip_amount,
                ..
            } => *skip_amount == 0,
            Command::Loop { id: _, body, .. } => {
                let init_rel_pointer = *rel_pointer;
                for loop_cmd in body {
                    if !check_loop_pointer_rec(loop_cmd, rel_pointer) {
                        return false;
                    }
                }
                init_rel_pointer == *rel_pointer
            }
            _ => true,
        }
    }

    check_loop_pointer_rec(command, &mut 0)
}

fn add_prev_value(
    idx: usize,
    tape: &HashMap<usize, AbstractCell>,
    prev_values: &mut HashMap<usize, u8>,
    should_add: bool,
) {
    if !should_add {
        return;
    }
    if let Some(cell) = tape.get(&idx) {
        if let AbstractCell::Value(cell_value) = cell {
            prev_values.entry(idx).or_insert(*cell_value);
        }
    } else {
        prev_values.entry(idx).or_insert(0);
    }
}

fn step(
    command: &Command,
    tape: &mut HashMap<usize, AbstractCell>,
    pointer: &mut usize,
    prev_values: &mut HashMap<usize, u8>,
    cmd_buf: &mut Vec<IOCommand>,
    inside_loop: bool,
) -> Result<Option<()>, String> {
    match command {
        Command::IncPointer { amount, .. } => {
            *pointer += *amount;
            Ok(None)
        }
        Command::DecPointer { amount, .. } => {
            *pointer -= *amount;
            Ok(None)
        }
        Command::IncData { offset, amount, .. } => {
            add_prev_value(
                pointer.wrapping_add_signed(*offset),
                tape,
                prev_values,
                inside_loop,
            );
            match tape
                .get(&pointer.wrapping_add_signed(*offset))
                .unwrap_or(&AbstractCell::Value(0))
            {
                AbstractCell::Value(cell_val) => {
                    tape.insert(
                        pointer.wrapping_add_signed(*offset),
                        AbstractCell::Value(*cell_val + amount),
                    );
                    Ok(None)
                }
                AbstractCell::Top => Ok(Some(())),
            }
        }
        Command::DecData { offset, amount, .. } => {
            add_prev_value(
                pointer.wrapping_add_signed(*offset),
                tape,
                prev_values,
                inside_loop,
            );
            match tape
                .get(&pointer.wrapping_add_signed(*offset))
                .unwrap_or(&AbstractCell::Value(0))
            {
                AbstractCell::Value(cell_val) => {
                    tape.insert(
                        pointer.wrapping_add_signed(*offset),
                        AbstractCell::Value(*cell_val - amount),
                    );
                    Ok(None)
                }
                AbstractCell::Top => Ok(Some(())),
            }
        }
        Command::SetData { offset, value, .. } => {
            add_prev_value(
                pointer.wrapping_add_signed(*offset),
                tape,
                prev_values,
                inside_loop,
            );
            tape.insert(
                pointer.wrapping_add_signed(*offset),
                AbstractCell::Value(*value),
            );
            Ok(None)
        }
        Command::Scan {
            id: _,
            direction,
            skip_amount,
            ..
        } => {
            let old_ptr = *pointer;
            loop {
                match tape.get(&pointer).unwrap_or(&AbstractCell::Value(0)) {
                    AbstractCell::Value(cell_val) => {
                        if *cell_val == 0 {
                            break Ok(None);
                        }
                        match direction {
                            Direction::Left => *pointer -= *skip_amount,
                            Direction::Right => *pointer += *skip_amount,
                        }
                    }
                    AbstractCell::Top => {
                        *pointer = old_ptr;
                        break Err("Encountered top when scanning".to_string());
                    }
                }
            }
        }
        Command::AddOffsetData {
            dest_offset,
            src_offset,
            multiplier,
            inverted,
            ..
        } => {
            add_prev_value(
                pointer.wrapping_add_signed(*dest_offset),
                tape,
                prev_values,
                inside_loop,
            );
            add_prev_value(
                pointer.wrapping_add_signed(*src_offset),
                tape,
                prev_values,
                inside_loop,
            );
            match tape
                .get(&pointer.wrapping_add_signed(*src_offset))
                .unwrap_or(&AbstractCell::Value(0))
            {
                AbstractCell::Value(src_val) => match tape
                    .get(&pointer.wrapping_add_signed(*dest_offset))
                    .unwrap_or(&AbstractCell::Value(0))
                {
                    AbstractCell::Value(dest_val) => {
                        let mut rhs = if *inverted {
                            0u8.wrapping_sub(*src_val) as usize
                        } else {
                            *src_val as usize
                        };
                        rhs = rhs.wrapping_mul(*multiplier) % 256;
                        tape.insert(
                            pointer.wrapping_add_signed(*dest_offset),
                            AbstractCell::Value(dest_val.wrapping_add(rhs as u8)),
                        );
                        Ok(None)
                    }
                    AbstractCell::Top => Ok(Some(())),
                },
                AbstractCell::Top => Ok(Some(())),
            }
        }
        Command::SubOffsetData {
            dest_offset,
            src_offset,
            multiplier,
            inverted,
            ..
        } => {
            add_prev_value(
                pointer.wrapping_add_signed(*dest_offset),
                tape,
                prev_values,
                inside_loop,
            );
            add_prev_value(
                pointer.wrapping_add_signed(*src_offset),
                tape,
                prev_values,
                inside_loop,
            );
            match tape
                .get(&pointer.wrapping_add_signed(*src_offset))
                .unwrap_or(&AbstractCell::Value(0))
            {
                AbstractCell::Value(src_val) => match tape
                    .get(&pointer.wrapping_add_signed(*dest_offset))
                    .unwrap_or(&AbstractCell::Value(0))
                {
                    AbstractCell::Value(dest_val) => {
                        let mut rhs = if *inverted {
                            0u8.wrapping_sub(*src_val) as usize
                        } else {
                            *src_val as usize
                        };
                        rhs = rhs.wrapping_mul(*multiplier) % 256;
                        tape.insert(
                            pointer.wrapping_add_signed(*dest_offset),
                            AbstractCell::Value(dest_val.wrapping_sub(rhs as u8)),
                        );
                        Ok(None)
                    }
                    AbstractCell::Top => Ok(Some(())),
                },
                AbstractCell::Top => Ok(Some(())),
            }
        }
        Command::Output { out_type, .. } => match out_type {
            OutputType::Const(_) => {
                cmd_buf.push(IOCommand {
                    command: command.clone(),
                    pointer: *pointer,
                });
                Ok(None)
            }
            OutputType::Cell { offset } => match tape
                .get(&pointer.wrapping_add_signed(*offset))
                .unwrap_or(&AbstractCell::Value(0))
            {
                AbstractCell::Value(cell_val) => {
                    add_prev_value(
                        pointer.wrapping_add_signed(*offset),
                        tape,
                        prev_values,
                        inside_loop,
                    );
                    cmd_buf.push(IOCommand {
                        command: Command::Output {
                            out_type: OutputType::Const(*cell_val),
                            count: 0,
                        },
                        pointer: *pointer,
                    });
                    Ok(None)
                }
                AbstractCell::Top => {
                    cmd_buf.push(IOCommand {
                        command: command.clone(),
                        pointer: *pointer,
                    });
                    Ok(None)
                }
            },
        },
        Command::Input { offset, .. } => {
            add_prev_value(
                pointer.wrapping_add_signed(*offset),
                tape,
                prev_values,
                true,
            );
            tape.insert(pointer.wrapping_add_signed(*offset), AbstractCell::Top);
            cmd_buf.push(IOCommand {
                command: command.clone(),
                pointer: *pointer,
            });
            Ok(None)
        }
        Command::Loop { id: _, body, .. } => {
            match tape.get(pointer).unwrap_or(&AbstractCell::Value(0)) {
                AbstractCell::Value(_) => {
                    let old_ptr = *pointer;
                    let mut ret_val = None;
                    loop {
                        add_prev_value(*pointer, tape, prev_values, true);
                        match tape.get(pointer).unwrap_or(&AbstractCell::Value(0)) {
                            AbstractCell::Value(curr_cell_val) => {
                                if *curr_cell_val == 0 {
                                    break Ok(ret_val);
                                }
                                for loop_cmd in body {
                                    match step(loop_cmd, tape, pointer, prev_values, cmd_buf, true)
                                    {
                                        Ok(ok_val) => match ok_val {
                                            Some(_) => ret_val = Some(()),
                                            None => (),
                                        },
                                        Err(e) => {
                                            *pointer = old_ptr;
                                            return Err(e);
                                        }
                                    }
                                }
                            }
                            AbstractCell::Top => {
                                if check_loop_pointer(command) {
                                    for loop_cmd in body {
                                        if let Err(e) = step_uncertain(
                                            loop_cmd,
                                            tape,
                                            pointer,
                                            prev_values,
                                            cmd_buf,
                                            true,
                                        ) {
                                            *pointer = old_ptr;
                                            return Err(e);
                                        }
                                    }
                                    return Ok(Some(()));
                                } else {
                                    *pointer = old_ptr;
                                    return Err(
                                        "Unbalanced pointer movement with input induction variable"
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }
                }
                AbstractCell::Top => {
                    if check_loop_pointer(command) {
                        let old_ptr = *pointer;
                        for loop_cmd in body {
                            if let Err(e) =
                                step_uncertain(loop_cmd, tape, pointer, prev_values, cmd_buf, true)
                            {
                                *pointer = old_ptr;
                                return Err(e);
                            }
                        }
                        Ok(Some(()))
                    } else {
                        Err("Unbalanced pointer movement with input induction variable".to_string())
                    }
                }
            }
        }
    }
}

fn step_uncertain(
    command: &Command,
    tape: &mut HashMap<usize, AbstractCell>,
    pointer: &mut usize,
    prev_values: &mut HashMap<usize, u8>,
    cmd_buf: &mut Vec<IOCommand>,
    inside_loop: bool,
) -> Result<(), String> {
    match command {
        Command::IncPointer { amount, .. } => {
            *pointer += *amount;
            Ok(())
        }
        Command::DecPointer { amount, .. } => {
            *pointer -= *amount;
            Ok(())
        }
        Command::IncData { offset, .. } => {
            add_prev_value(
                pointer.wrapping_add_signed(*offset),
                tape,
                prev_values,
                true,
            );
            tape.insert(pointer.wrapping_add_signed(*offset), AbstractCell::Top);
            Ok(())
        }
        Command::DecData { offset, .. } => {
            add_prev_value(
                pointer.wrapping_add_signed(*offset),
                tape,
                prev_values,
                true,
            );
            tape.insert(pointer.wrapping_add_signed(*offset), AbstractCell::Top);
            Ok(())
        }
        Command::SetData { offset, .. } => {
            add_prev_value(
                pointer.wrapping_add_signed(*offset),
                tape,
                prev_values,
                true,
            );
            // This may be able to be relaxed
            tape.insert(pointer.wrapping_add_signed(*offset), AbstractCell::Top);
            Ok(())
        }
        Command::Scan {
            id: _,
            direction,
            skip_amount,
            ..
        } => {
            let old_ptr = *pointer;
            loop {
                match tape.get(&pointer).unwrap_or(&AbstractCell::Value(0)) {
                    AbstractCell::Value(cell_val) => {
                        if *cell_val == 0 {
                            break Ok(());
                        }
                        match direction {
                            Direction::Left => *pointer -= *skip_amount,
                            Direction::Right => *pointer += *skip_amount,
                        }
                    }
                    AbstractCell::Top => {
                        *pointer = old_ptr;
                        break Err("Encountered top when scanning in uncertain".to_string());
                    }
                }
            }
        }
        Command::AddOffsetData {
            dest_offset,
            src_offset,
            ..
        } => {
            add_prev_value(
                pointer.wrapping_add_signed(*dest_offset),
                tape,
                prev_values,
                true,
            );
            add_prev_value(
                pointer.wrapping_add_signed(*src_offset),
                tape,
                prev_values,
                true,
            );
            tape.insert(pointer.wrapping_add_signed(*dest_offset), AbstractCell::Top);
            Ok(())
        }
        Command::SubOffsetData {
            dest_offset,
            src_offset,
            ..
        } => {
            add_prev_value(
                pointer.wrapping_add_signed(*dest_offset),
                tape,
                prev_values,
                true,
            );
            add_prev_value(
                pointer.wrapping_add_signed(*src_offset),
                tape,
                prev_values,
                true,
            );
            tape.insert(pointer.wrapping_add_signed(*dest_offset), AbstractCell::Top);
            Ok(())
        }
        Command::Output { out_type, .. } => match out_type {
            OutputType::Const(_) => Ok(()),
            OutputType::Cell { offset } => {
                add_prev_value(
                    pointer.wrapping_add_signed(*offset),
                    tape,
                    prev_values,
                    true,
                );
                Ok(())
            }
        },
        Command::Input { offset, .. } => {
            add_prev_value(
                pointer.wrapping_add_signed(*offset),
                tape,
                prev_values,
                true,
            );
            tape.insert(pointer.wrapping_add_signed(*offset), AbstractCell::Top);
            Ok(())
        }
        Command::Loop { id: _, body, .. } => {
            if check_loop_pointer(command) {
                add_prev_value(*pointer, tape, prev_values, true);
                let old_ptr = *pointer;
                for loop_cmd in body {
                    if let Err(e) =
                        step_uncertain(loop_cmd, tape, pointer, prev_values, cmd_buf, true)
                    {
                        *pointer = old_ptr;
                        return Err(e);
                    }
                }
                Ok(())
            } else {
                Err(
                    "Unbalanced pointer movement with input induction variable in uncertain"
                        .to_string(),
                )
            }
        }
    }
}

pub fn partial_eval(cmds: &Vec<Command>) -> Vec<Command> {
    let mut new_cmds: Vec<Command> = vec![];

    let mut tape: HashMap<usize, AbstractCell> = HashMap::new();
    let mut abstract_pointer = INIT_POINTER_LOC;
    let mut pointer = INIT_POINTER_LOC;
    let mut prev_values: HashMap<usize, u8> = HashMap::new();
    let mut cmd_buf: Vec<IOCommand> = vec![];

    let mut error_occurred = false;
    for cmd in cmds {
        let prev_pointer = pointer;
        if !error_occurred {
            let res = step(
                &cmd,
                &mut tape,
                &mut pointer,
                &mut prev_values,
                &mut cmd_buf,
                false,
            );
            match res {
                Ok(ok_val) => match ok_val {
                    Some(_) => {
                        if prev_values.len() > 0 {
                            for (key, value) in &prev_values {
                                new_cmds.push(Command::SetData {
                                    offset: (*key as isize) - (abstract_pointer as isize),
                                    value: *value,
                                    count: 0,
                                })
                            }
                        }
                        let prev_pointer_diff =
                            (prev_pointer as isize) - (abstract_pointer as isize);
                        if prev_pointer_diff > 0 {
                            new_cmds.push(Command::IncPointer {
                                amount: prev_pointer_diff as usize,
                                count: 0,
                            });
                        } else if prev_pointer_diff < 0 {
                            new_cmds.push(Command::DecPointer {
                                amount: -prev_pointer_diff as usize,
                                count: 0,
                            });
                        }
                        prev_values.clear();
                        cmd_buf.clear();
                        new_cmds.push(cmd.clone());
                        let pointer_diff = (pointer as isize) - (abstract_pointer as isize);
                        if pointer_diff > 0 {
                            new_cmds.push(Command::IncPointer {
                                amount: pointer_diff as usize,
                                count: 0,
                            });
                        } else if pointer_diff < 0 {
                            new_cmds.push(Command::DecPointer {
                                amount: -pointer_diff as usize,
                                count: 0,
                            });
                        }
                        abstract_pointer = pointer;
                    }
                    None => {
                        if cmd_buf.len() > 0 {
                            if prev_values.len() > 0 {
                                for (key, value) in &prev_values {
                                    new_cmds.push(Command::SetData {
                                        offset: (*key as isize) - (abstract_pointer as isize),
                                        value: *value,
                                        count: 0,
                                    })
                                }
                                prev_values.clear();
                            }
                            for buf_cmd in &cmd_buf {
                                let pointer_diff =
                                    (buf_cmd.pointer as isize) - (abstract_pointer as isize);
                                if pointer_diff > 0 {
                                    new_cmds.push(Command::IncPointer {
                                        amount: pointer_diff as usize,
                                        count: 0,
                                    });
                                } else if pointer_diff < 0 {
                                    new_cmds.push(Command::DecPointer {
                                        amount: -pointer_diff as usize,
                                        count: 0,
                                    });
                                }
                                abstract_pointer = buf_cmd.pointer;
                                new_cmds.push(buf_cmd.command.clone());
                            }
                        }
                        prev_values.clear();
                        cmd_buf.clear();
                    }
                },
                Err(_e) => {
                    error_occurred = true;
                    //eprintln!("Error in partial evaulation: {e}");
                    for (key, value) in &prev_values {
                        tape.insert(*key, AbstractCell::Value(*value));
                    }
                    for (key, value) in &tape {
                        if let AbstractCell::Value(cell_val) = value {
                            new_cmds.push(Command::SetData {
                                offset: (*key as isize) - (abstract_pointer as isize),
                                value: *cell_val,
                                count: 0,
                            })
                        }
                    }
                    let pointer_diff = (prev_pointer as isize) - (abstract_pointer as isize);
                    if pointer_diff > 0 {
                        new_cmds.push(Command::IncPointer {
                            amount: pointer_diff as usize,
                            count: 0,
                        });
                    } else if pointer_diff < 0 {
                        new_cmds.push(Command::DecPointer {
                            amount: -pointer_diff as usize,
                            count: 0,
                        });
                    }
                    abstract_pointer = prev_pointer;
                    new_cmds.push(cmd.clone());
                }
            }
        } else {
            new_cmds.push(cmd.clone());
        }
    }

    return new_cmds;
}
