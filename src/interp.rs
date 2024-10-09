use crate::parser::{Command, Direction, OutputType};

const INIT_TAPE_SIZE: usize = 0x100000;
const INIT_POINTER_LOC: usize = 0x400;

pub fn interp(commands: &mut [Command]) {
    fn interp_rec(
        commands: &mut [Command],
        tape: &mut [u8],
        pointer: &mut usize,
        pc: &mut usize,
    ) -> std::io::Result<()> {
        while *pc < commands.len() {
            match &mut commands[*pc] {
                Command::IncPointer {
                    amount,
                    ref mut count,
                } => {
                    *count += 1;
                    *pointer += *amount;
                }
                Command::DecPointer {
                    amount,
                    ref mut count,
                } => {
                    *count += 1;
                    *pointer -= *amount;
                }
                Command::IncData {
                    offset,
                    amount,
                    ref mut count,
                } => {
                    *count += 1;
                    tape[pointer.wrapping_add_signed(*offset)] =
                        tape[pointer.wrapping_add_signed(*offset)].wrapping_add(*amount);
                }
                Command::DecData {
                    offset,
                    amount,
                    ref mut count,
                } => {
                    *count += 1;
                    tape[pointer.wrapping_add_signed(*offset)] =
                        tape[pointer.wrapping_add_signed(*offset)].wrapping_sub(*amount);
                }
                Command::SetData {
                    offset,
                    value,
                    ref mut count,
                } => {
                    *count += 1;
                    tape[pointer.wrapping_add_signed(*offset)] = *value;
                }
                Command::Scan {
                    id: _,
                    direction,
                    skip_amount,
                    ref mut count,
                } => {
                    *count += 1;
                    while tape[*pointer] != 0 {
                        match direction {
                            Direction::Left => *pointer -= *skip_amount,
                            Direction::Right => *pointer += *skip_amount,
                        }
                    }
                }
                Command::AddOffsetData {
                    dest_offset,
                    src_offset,
                    multiplier,
                    inverted,
                    ref mut count,
                } => {
                    *count += 1;
                    let mut src_val = if *inverted {
                        0u8.wrapping_sub(tape[pointer.wrapping_add_signed(*src_offset)]) as usize
                    } else {
                        tape[pointer.wrapping_add_signed(*src_offset)] as usize
                    };
                    src_val = src_val.wrapping_mul(*multiplier) % 256;
                    tape[pointer.wrapping_add_signed(*dest_offset)] =
                        tape[pointer.wrapping_add_signed(*dest_offset)].wrapping_add(src_val as u8);
                }
                Command::SubOffsetData {
                    dest_offset,
                    src_offset,
                    multiplier,
                    inverted,
                    ref mut count,
                } => {
                    *count += 1;
                    let mut src_val = if *inverted {
                        0u8.wrapping_sub(tape[pointer.wrapping_add_signed(*src_offset)]) as usize
                    } else {
                        tape[pointer.wrapping_add_signed(*src_offset)] as usize
                    };
                    src_val = src_val.wrapping_mul(*multiplier) % 256;
                    tape[pointer.wrapping_add_signed(*dest_offset)] =
                        tape[pointer.wrapping_add_signed(*dest_offset)].wrapping_sub(src_val as u8);
                }
                Command::Output {
                    out_type,
                    ref mut count,
                } => {
                    use std::io::Write;

                    *count += 1;
                    let buf: Vec<u8>;
                    match out_type {
                        OutputType::Const(val) => buf = vec![*val],
                        OutputType::Cell { offset } => {
                            buf = vec![tape[pointer.wrapping_add_signed(*offset)]]
                        }
                    }
                    std::io::stdout().write_all(&buf)?;
                }
                Command::Input {
                    offset,
                    ref mut count,
                } => {
                    use std::io::Read;

                    *count += 1;
                    let mut input_buf: [u8; 1] = [0; 1];
                    if let Err(..) = std::io::stdin().read_exact(&mut input_buf) {
                        tape[pointer.wrapping_add_signed(*offset)] = 255; // -1
                    } else {
                        tape[pointer.wrapping_add_signed(*offset)] = input_buf[0];
                    }
                }
                Command::Loop {
                    body,
                    id: _,
                    ref mut start_count,
                    ref mut end_count,
                } => {
                    *start_count += 1;
                    while tape[*pointer] != 0 {
                        let mut loop_pc = 0;
                        if let Err(e) = interp_rec(body, tape, pointer, &mut loop_pc) {
                            eprintln!("{}", e);
                        }

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
        Ok(())
    }
    let mut tape: Vec<u8> = vec![0; INIT_TAPE_SIZE];
    let mut pointer = INIT_POINTER_LOC;
    let mut pc = 0;
    if let Err(e) = interp_rec(commands, &mut tape, &mut pointer, &mut pc) {
        eprintln!("{}", e);
    };
}
