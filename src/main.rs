mod parser;
mod profiler;

use std::io::{self, Write};

const INIT_TAPE_SIZE: usize = 65536;
const INIT_POINTER_LOC: usize = INIT_TAPE_SIZE / 2;

struct Args {
    filename: String,
    enable_profiler: bool,
    enable_pretty_print: bool, // This disables interpretation
}

fn parse_args() -> Args {
    let mut tmp_filename = None;
    let mut args_struct = Args {
        filename: "".to_string(),
        enable_profiler: false,
        enable_pretty_print: false,
    };
    for arg in std::env::args().skip(1).peekable() {
        match arg.as_str() {
            "-p" => args_struct.enable_profiler = true,
            "--pretty" => args_struct.enable_pretty_print = true,
            _ => tmp_filename = Some(arg),
        }
    }

    // Requires filename
    match tmp_filename {
        Some(val) => args_struct.filename = val,
        None => {
            eprintln!("Usage: bfr <filename> [-p] [--pretty]");
            std::process::exit(1);
        }
    }
    
    args_struct
}


fn interp(commands: &mut [parser::Command], tape: &mut [u8], pointer: &mut usize, pc: &mut usize) {
    use parser::Command;

    while *pc < commands.len() {
        match &mut commands[*pc] {
            Command::IncPointer { ref mut count } => {
                *count += 1;
                *pointer += 1;
            },
            Command::DecPointer { ref mut count } => {
                *count += 1;
                *pointer -= 1;
            },
            Command::IncData { ref mut count } => {
                *count += 1;
                tape[*pointer] = tape[*pointer].wrapping_add(1);
            },
            Command::DecData { ref mut count } => {
                *count += 1;
                tape[*pointer] = tape[*pointer].wrapping_sub(1);
            },
            Command::Output { ref mut count } => {
                *count += 1;
                match char::from_u32(u32::from(tape[*pointer])) {
                    Some(c) => print!("{}", c),
                    None => {},
                }
            }
            Command::Input { ref mut count } => {
                use std::io::Read;

                *count += 1;
                let mut input_buf: [u8; 1] = [0; 1];
                std::io::stdin().read_exact(&mut input_buf).expect("Failed to read input");
                tape[*pointer] = input_buf[0];
            },
            Command::Loop { body, id: _, ref mut start_count, ref mut end_count } => {
                *start_count += 1;
                while tape[*pointer] != 0 {
                    let mut loop_pc = 0;
                    interp(body, tape, pointer, &mut loop_pc);

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
}

fn write_assembly_header(out_file: &mut std::fs::File) {
    writeln!(*out_file, r#"
.section .text

.globl _start

_start:
    pushq %rbp        # Save previous base pointer
    movq  %rsp, %rbp  # Set new base pointer

    call main

    # Exit after returning from main
    movq  $60,  %rax  # Exit system call number
    xorq  %rdi, %rdi  # Exit status 0
    syscall           # Make exit system call

main:
    pushq %rbp
    movq  %rsp, %rbp

    # Allocate memory using mmap
    movq $9,        %rax  # Mmap system call number
    movq $0,        %rdi  # Address (NULL for system to choose)
    movq $0x200000, %rsi  # Allocate 2 MB
    movq $3,        %rdx  # PROT_READ | PROT_WRITE
    movq $0x22,     %r10  # MAP_PRIVATE | MAP_ANONYMOUS
    movq $-1,       %r8   # Fd (-1 for anonymous mapping)
    movq $0,        %r9   # Offset (0 for anonymous mapping)
    syscall               # Make mmap system call

    # TODO: Check if mmap failed

    movq %rax,      %r12 # Move tape address into callee saved register
    addq $0x100000, %r12 # Move the pointer to the middle of the tape

    # Begin program code
    "#);
}

fn write_assembly_footer(out_file: &mut std::fs::File) {
    writeln!(*out_file, r#"
    # Should be at the bottom of main
    
    # Unmap memory from mmap
    movq $11,       %rax # Munmap system call number
    movq %r12,      %rdi # Address
    movq $0x200000, %rsi # 2 MB size
    syscall

    # TODO: Check if munmap failed

    movq  %rbp, %rsp
    popq  %rbp

    # Return to _start
    ret
    "#);
}

fn compile(out_file: &mut std::fs::File, commands: &[parser::Command]) {
    use parser::Command;

    fn compile_rec(out_file: &mut std::fs::File, commands: &[parser::Command]) {
        for command in commands {
            match command {
                Command::IncPointer { .. } => {
                    
                },
                Command::DecPointer { .. } => {

                },
                Command::IncData { .. } => {

                },
                Command::DecData { .. } => {

                },
                Command::Output { .. } => {
                    //*count += 1;
                    //match char::from_u32(u32::from(tape[*pointer])) {
                    //    Some(c) => print!("{}", c),
                    //    None => {},
                    //}
                }
                Command::Input { .. } => {
                    //use std::io::Read;
                    //
                    //*count += 1;
                    //let mut input_buf: [u8; 1] = [0; 1];
                    //std::io::stdin().read_exact(&mut input_buf).expect("Failed to read input");
                    //tape[*pointer] = input_buf[0];
                },
                Command::Loop { body, id, .. } => {
                    compile_rec(out_file, body);
                }
            }
        }
    }

    write_assembly_header(out_file);
    compile_rec(out_file, commands);
    write_assembly_footer(out_file);
}

fn main() {

    let args = parse_args();

    // Read the file contents
    let src_contents = match std::fs::read_to_string(&args.filename) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading file {}: {}", args.filename, e);
            std::process::exit(1);
        }
    };

    let mut out_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("bfr.s")
        .expect("Unable to open output file");

    let mut commands = parser::parse(&src_contents);

    if args.enable_pretty_print {
        parser::pretty_print(&commands);
        return;
    }

    compile(&mut out_file, &commands);

    //let mut tape: Vec<u8> = vec![0; INIT_TAPE_SIZE];
    //let mut pointer = INIT_POINTER_LOC;
    //let mut pc = 0;
    //
    //interp(&mut commands, &mut tape, &mut pointer, &mut pc);
    //
    //println!();
    //println!("Terminated normally");
    //
    //if args.enable_profiler {
    //    profiler::print_profile(&commands);
    //}
}
