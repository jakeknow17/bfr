use crate::parser::{Command, Direction, OutputType};
use std::collections::HashSet;

#[derive(Debug, Clone)]
enum AbstractCell {
    Value(u8),
    Top,
}

#[derive(Debug, Clone)]
#[allow(unused)]
enum AbstractPointer {
    Value(usize),
    Top,
}

#[derive(Debug, Clone)]
enum IO {
    In { offset: isize },
    Out(u8),
}

const INIT_TAPE_SIZE: usize = 0x100000;
const INIT_POINTER_LOC: usize = 0x400;

fn transfer(
    command: &Command,
    tape: &mut Vec<AbstractCell>,
    pointer: &mut AbstractPointer,
    modified_cells: &mut HashSet<usize>,
    io_buf: &mut Vec<IO>,
) -> Result<(), ()> {
    match command {
        Command::IncPointer { amount, .. } => match pointer {
            AbstractPointer::Value(ref mut val) => {
                *val += *amount;
                Ok(())
            }
            AbstractPointer::Top => Err(()),
        },
        Command::DecPointer { amount, .. } => match pointer {
            AbstractPointer::Value(ref mut val) => {
                *val -= *amount;
                Ok(())
            }
            AbstractPointer::Top => Err(()),
        },
        Command::IncData { offset, amount, .. } => match pointer {
            AbstractPointer::Value(val) => match tape[val.wrapping_add_signed(*offset)] {
                AbstractCell::Value(ref mut cell_val) => {
                    *cell_val += amount;
                    modified_cells.insert(val.wrapping_add_signed(*offset));
                    Ok(())
                }
                AbstractCell::Top => Err(()),
            },
            AbstractPointer::Top => Err(()),
        },
        Command::DecData { offset, amount, .. } => match pointer {
            AbstractPointer::Value(val) => match tape[val.wrapping_add_signed(*offset)] {
                AbstractCell::Value(ref mut cell_val) => {
                    *cell_val -= amount;
                    modified_cells.insert(val.wrapping_add_signed(*offset));
                    Ok(())
                }
                AbstractCell::Top => Err(()),
            },
            AbstractPointer::Top => Err(()),
        },
        Command::SetData { offset, value, .. } => match pointer {
            AbstractPointer::Value(val) => {
                tape[val.wrapping_add_signed(*offset)] = AbstractCell::Value(*value);
                modified_cells.insert(val.wrapping_add_signed(*offset));
                Ok(())
            }
            AbstractPointer::Top => Err(()),
        },
        Command::Scan {
            direction,
            skip_amount,
            ..
        } => {
            let old_ptr = pointer.clone();
            loop {
                match pointer {
                    AbstractPointer::Value(ref mut val) => match tape[*val] {
                        AbstractCell::Value(cell_val) => {
                            if cell_val == 0 {
                                break Ok(());
                            }
                            match direction {
                                Direction::Left => *val -= *skip_amount,
                                Direction::Right => *val += *skip_amount,
                            }
                        }
                        AbstractCell::Top => {
                            *pointer = old_ptr;
                            break Err(());
                        }
                    },
                    AbstractPointer::Top => {
                        *pointer = old_ptr;
                        break Err(());
                    }
                };
            }
        }
        Command::AddOffsetData {
            dest_offset,
            src_offset,
            multiplier,
            inverted,
            ..
        } => match pointer {
            AbstractPointer::Value(val) => match tape[val.wrapping_add_signed(*src_offset)] {
                AbstractCell::Value(src_val) => match tape[val.wrapping_add_signed(*dest_offset)] {
                    AbstractCell::Value(ref mut dest_val) => {
                        let mut rhs = if *inverted {
                            0u8.wrapping_sub(src_val) as usize
                        } else {
                            src_val as usize
                        };
                        rhs = rhs.wrapping_mul(*multiplier) % 256;
                        *dest_val = dest_val.wrapping_add(rhs as u8);
                        Ok(())
                    }
                    AbstractCell::Top => Err(()),
                },
                AbstractCell::Top => Err(()),
            },
            AbstractPointer::Top => Err(()),
        },
        Command::SubOffsetData {
            dest_offset,
            src_offset,
            multiplier,
            inverted,
            ..
        } => match pointer {
            AbstractPointer::Value(val) => match tape[val.wrapping_add_signed(*src_offset)] {
                AbstractCell::Value(src_val) => match tape[val.wrapping_add_signed(*dest_offset)] {
                    AbstractCell::Value(ref mut dest_val) => {
                        let mut rhs = if *inverted {
                            0u8.wrapping_sub(src_val) as usize
                        } else {
                            src_val as usize
                        };
                        rhs = rhs.wrapping_mul(*multiplier) % 256;
                        *dest_val = dest_val.wrapping_sub(rhs as u8);
                        Ok(())
                    }
                    AbstractCell::Top => Err(()),
                },
                AbstractCell::Top => Err(()),
            },
            AbstractPointer::Top => Err(()),
        },
        Command::Output { out_type, .. } => match out_type {
            OutputType::Const(val) => {
                io_buf.push(IO::Out(*val));
                Ok(())
            }
            OutputType::Cell { offset } => match pointer {
                AbstractPointer::Value(val) => match tape[val.wrapping_add_signed(*offset)] {
                    AbstractCell::Value(cell_val) => {
                        io_buf.push(IO::Out(cell_val));
                        Ok(())
                    }
                    AbstractCell::Top => Err(()),
                },
                AbstractPointer::Top => Err(()),
            },
        },
        Command::Input { offset, .. } => match pointer {
            AbstractPointer::Value(val) => {
                tape[val.wrapping_add_signed(*offset)] = AbstractCell::Top;
                // Since modified to Top, don't add to modified_cells
                io_buf.push(IO::In {
                    offset: (*val as isize) - (INIT_POINTER_LOC as isize),
                });
                Ok(())
            }
            AbstractPointer::Top => Err(()),
        },
        Command::Loop { body, .. } => {
            //let old_ptr = pointer.clone();
            //let old_tape = tape.clone();
            loop {
                match pointer {
                    AbstractPointer::Value(ref mut val) => match tape[*val] {
                        AbstractCell::Value(cell_val) => {
                            if cell_val == 0 {
                                break Ok(());
                            }
                            for body_cmd in body {
                                let res =
                                    transfer(&body_cmd, tape, pointer, modified_cells, io_buf);
                                if let Err(e) = res {
                                    //*pointer = old_ptr;
                                    //*tape = old_tape;
                                    return Err(e);
                                }
                            }
                        }
                        AbstractCell::Top => {
                            //*pointer = old_ptr;
                            //*tape = old_tape;
                            break Err(());
                        }
                    },
                    AbstractPointer::Top => {
                        //*pointer = old_ptr;
                        //*tape = old_tape;
                        break Err(());
                    }
                };
            }
        }
    }
}

fn contains_input(cmds: &[Command]) -> bool {
    let mut input_found = false;
    for cmd in cmds {
        match cmd {
            Command::Loop { body, .. } => {
                input_found |= contains_input(body);
            }
            Command::Input { .. } => {
                input_found = true;
            }
            _ => (),
        }
    }
    input_found
}

pub fn partial_eval(cmds: &mut Vec<Command>) {
    let mut tape: Vec<AbstractCell> = vec![AbstractCell::Value(0); INIT_TAPE_SIZE];
    let mut pointer = AbstractPointer::Value(INIT_POINTER_LOC);
    let mut modified_cells: HashSet<usize> = HashSet::new();
    let mut io_buf: Vec<IO> = vec![];

    let mut stopped_early = false;
    let mut i = 0;
    while i < cmds.len() {
        let cmd = &cmds[i];
        match cmd {
            Command::Input { .. } => {
                stopped_early = true;
                break;
            }
            Command::Loop { body, .. } => {
                if contains_input(body) {
                    stopped_early = true;
                    break;
                }
            }
            _ => (),
        }

        let res = transfer(
            cmd,
            &mut tape,
            &mut pointer,
            &mut modified_cells,
            &mut io_buf,
        );
        if let Err(_) = res {
            eprintln!("Cannot fully run the partial evaluator");
            stopped_early = true;
            break;
        }
        i += 1;
    }

    // Use results from partial evaluation
    cmds.drain(0..i);
    let mut new_cmds: Vec<Command> = vec![];
    let mut includes_input = false;
    for io_cmd in io_buf {
        match io_cmd {
            IO::In { offset } => {
                includes_input = true;
                new_cmds.push(Command::Input { offset, count: 0 })
            }
            IO::Out(val) => new_cmds.push(Command::Output {
                out_type: OutputType::Const(val),
                count: 0,
            }),
        }
    }
    if stopped_early || includes_input {
        for cell in modified_cells {
            match tape[cell] {
                AbstractCell::Value(val) => new_cmds.push(Command::SetData {
                    offset: (cell as isize) - (INIT_POINTER_LOC as isize),
                    value: val,
                    count: 0,
                }),
                AbstractCell::Top => panic!("Bad partial evaluation pass"),
            }
        }
    }
    if let AbstractPointer::Value(ptr_val) = pointer {
        let ptr_offset = (ptr_val as isize) - (INIT_POINTER_LOC as isize);
        if ptr_offset >= 0 {
            new_cmds.push(Command::IncPointer {
                amount: ptr_offset as usize,
                count: 0,
            });
        } else {
            new_cmds.push(Command::DecPointer {
                amount: -ptr_offset as usize,
                count: 0,
            });
        }
    } else {
        panic!("Bad partial evaluation pass");
    }
    cmds.splice(0..0, new_cmds);
}
