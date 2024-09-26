mod optimizer;
mod parser;
mod profiler;

use clap::Parser;
use std::io::Write;

const INIT_TAPE_SIZE: usize = 0x200000;
const INIT_POINTER_LOC: usize = INIT_TAPE_SIZE / 2;

#[derive(Parser)]
#[command(name = "bfr")]
#[command(version = "1.0")]
#[command(about = "A simple Brainfuck interpreter written in Rust", long_about = None)]
struct Args {
    /// Source file
    file_name: String,

    /// Enable profiler. Also enables interpretation
    #[arg(short = 'p', long)]
    profile: bool,

    /// Pretty print parser output. Disables interpretation and compilation
    #[arg(short = 'P', long)]
    pretty_print: bool,

    /// Name of the output file
    #[arg(short, long = "output", value_name = "FILE", default_value_t = String::from("a.out"))]
    out_file: String,

    /// Interpret source file without compiling
    #[arg(short, long)]
    interp: bool,

    /// Output assembly file
    #[arg(short = 'S', long = "assembly")]
    output_asm: bool,

    /// Output object file
    #[arg(short = 'c', long = "object")]
    output_object: bool,

    /// Optimization level (0-3)
    #[arg(short = 'O', default_value_t = 1)]
    optimization_level: u8,
}

pub fn interp(commands: &mut [parser::Command]) {
    use parser::Command;

    fn interp_rec(
        commands: &mut [parser::Command],
        tape: &mut [u8],
        pointer: &mut usize,
        pc: &mut usize,
    ) {
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
                Command::AddOffsetData {
                    src_offset,
                    dest_list,
                    ref mut count,
                } => {
                    *count += 1;
                    let mut total: u8 = 1;
                    for dest in dest_list {
                        let tmp = (tape[pointer.wrapping_add_signed(dest.dst_offset)] as usize)
                            .wrapping_mul(dest.multiplier) as u8;
                        total = total.wrapping_mul(tmp);
                    }
                    tape[pointer.wrapping_add_signed(*src_offset)] =
                        tape[pointer.wrapping_add_signed(*src_offset)].wrapping_add(total);
                }
                Command::SubOffsetData {
                    src_offset,
                    dest_list,
                    ref mut count,
                } => {
                    *count += 1;
                    let mut total: u8 = 1;
                    for dest in dest_list {
                        let tmp = (tape[pointer.wrapping_add_signed(dest.dst_offset)] as usize)
                            .wrapping_mul(dest.multiplier) as u8;
                        total = total.wrapping_mul(tmp);
                    }
                    tape[pointer.wrapping_add_signed(*src_offset)] =
                        tape[pointer.wrapping_add_signed(*src_offset)].wrapping_sub(total);
                }
                Command::Output {
                    id: _,
                    ref mut count,
                } => {
                    *count += 1;
                    match char::from_u32(u32::from(tape[*pointer])) {
                        Some(c) => print!("{}", c),
                        None => {}
                    }
                }
                Command::Input {
                    id: _,
                    ref mut count,
                } => {
                    use std::io::Read;

                    *count += 1;
                    let mut input_buf: [u8; 1] = [0; 1];
                    std::io::stdin()
                        .read_exact(&mut input_buf)
                        .expect("Failed to read input");
                    tape[*pointer] = input_buf[0];
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
                        interp_rec(body, tape, pointer, &mut loop_pc);

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
    let mut tape: Vec<u8> = vec![0; INIT_TAPE_SIZE];
    let mut pointer = INIT_POINTER_LOC;
    let mut pc = 0;
    interp_rec(commands, &mut tape, &mut pointer, &mut pc);
}

pub fn replace_extension_filepath(filepath: &str, ext: &str) -> String {
    return if let Some(pos) = filepath.rfind('.') {
        format!("{}{}", &filepath[..pos], ext)
    } else {
        format!("{}{}", &filepath, ext)
    };
}

pub fn strip_directories_filepath(filepath: &str) -> &str {
    return if let Some(pos) = filepath.rfind('/') {
        &filepath[pos + 1..]
    } else {
        filepath
    };
}

fn assemble(asm_string: &str, object_filepath: &str) -> Result<(), String> {
    let mut as_process = std::process::Command::new("as")
        .arg("-o")
        .arg(object_filepath)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|_| {
            "Error: `as` assembler not found. Please ensure it is installed on your system."
                .to_string()
        })?;

    // Write the assembly code to the stdin of the `as` process
    if let Some(stdin) = as_process.stdin.as_mut() {
        stdin
            .write_all(asm_string.as_bytes())
            .map_err(|_| "Error: Failed to write assembly code to assembler.".to_string())?;
    } else {
        return Err("Error: Failed to open stdin for the assembler.".to_string());
    }

    let as_status = as_process
        .wait()
        .map_err(|_| "Error: Failed to wait for assembler process.".to_string())?;

    if !as_status.success() {
        return Err(format!(
            "Error: Assembler failed with exit code: {:?}",
            as_status.code()
        ));
    }

    Ok(())
}

fn link(object_filepath: &str, dest_file: &str, keep_object: bool) -> Result<(), String> {
    let ld_status = std::process::Command::new("ld")
        .arg("-o")
        .arg(dest_file)
        .arg(object_filepath)
        .spawn()
        .map_err(|_| {
            "Error: `ld` linker not found. Please ensure it is installed on your system."
                .to_string()
        })?
        .wait()
        .map_err(|_| "Error: Failed to wait for linker process.".to_string())?;

    if !ld_status.success() {
        return Err(format!(
            "Error: Linker failed with exit code: {:?}",
            ld_status.code()
        ));
    }

    // Delete the object file unless `keep_object` flag is set
    if !keep_object {
        std::fs::remove_file(object_filepath)
            .map_err(|_| format!("Warning: Failed to delete object file: {}", object_filepath))?;
    }

    Ok(())
}

fn append_assembly_header(out_string: &mut String) {
    out_string.push_str(
        r#"
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
    pushq %r12

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

    pushq %rax            # Save tape address to the stack
    movq  %rax,      %r12 # Move tape address into callee saved register
    addq  $0x100000, %r12 # Move the pointer to the middle of the tape

    # Begin program code
"#,
    );
}

fn append_assembly_footer(out_string: &mut String) {
    out_string.push_str(
        r#"    # At bottom of main
    
    # Unmap memory from mmap
    movq $11,       %rax # Munmap system call number
    movq %r12,      %rdi # Address
    movq $0x200000, %rsi # 2 MB size
    syscall

    # TODO: Check if munmap failed

    popq %rax
    popq %r12
    popq %rbp

    # Return to _start
    ret
"#,
    );
}

fn compile(
    commands: &[parser::Command],
    src_filepath: &str,
    dest_filename: &str,
    output_asm_file: bool,
    output_object_file: bool,
) {
    use parser::Command;

    fn append_pointer_op(
        out_string: &mut String,
        single_op: &str,
        multi_op: &str,
        amount: usize,
        reg: &str,
        comment: &str,
    ) {
        out_string.push_str(&format!("    # {}\n", comment));
        if amount == 1 {
            out_string.push_str(&format!("    {} {}\n", single_op, reg));
        } else {
            out_string.push_str(&format!("    {} ${}, {}\n", multi_op, amount, reg));
        }
        out_string.push('\n');
    }

    fn append_data_op(
        out_string: &mut String,
        offset: isize,
        amount: u8,
        single_op: &str,
        multi_op: &str,
        comment: &str,
    ) {
        let offset_str = if offset == 0 {
            String::from("")
        } else {
            format!("{}", offset)
        };
        out_string.push_str(&format!("    # {}\n", comment));
        out_string.push_str(&format!("    movb {}(%r12), %al\n", offset_str));
        if amount == 1 {
            out_string.push_str(&format!("    {} %al\n", single_op));
        } else {
            out_string.push_str(&format!("    {} ${}, %al\n", multi_op, amount));
        }
        out_string.push_str(&format!("    movb %al, {}(%r12)\n", offset_str));
        out_string.push('\n');
    }

    fn append_io_syscall(
        out_string: &mut String,
        syscall_num: i32,
        fd: i32,
        id: usize,
        comment: &str,
    ) {
        out_string.push_str(&format!(
            r#"    # {}
    movq ${}, %rax
    movq ${}, %rdi
    movq %r12, %rsi
    movq $1, %rdx
    syscall
"#,
            comment, syscall_num, fd
        ));

        // Read
        if syscall_num == 0 {
            out_string.push_str(&format!(
                r#"    testq %rax, %rax
    jnz   read_nonzero{}
    movb  $-1, (%r12)
    read_nonzero{}:
"#,
                id, id
            ));
        }
        out_string.push('\n');
    }

    fn compile_rec(out_string: &mut String, commands: &[parser::Command]) {
        for command in commands {
            match command {
                Command::IncPointer { amount, .. } => {
                    append_pointer_op(out_string, "incq", "addq", *amount, "%r12", ">");
                }
                Command::DecPointer { amount, .. } => {
                    append_pointer_op(out_string, "decq", "subq", *amount, "%r12", "<");
                }
                Command::IncData { offset, amount, .. } => {
                    append_data_op(out_string, *offset, *amount, "incb", "addb", "+");
                }
                Command::DecData { offset, amount, .. } => {
                    append_data_op(out_string, *offset, *amount, "decb", "subb", "-");
                }
                Command::SetData { offset, value, .. } => {
                    let offset_str = if *offset == 0 {
                        String::from("")
                    } else {
                        format!("{}", offset)
                    };
                    out_string.push_str(&format!(
                        "{:24} # ={}\n",
                        format!("    movb ${}, {}(%r12)", value, offset_str),
                        value
                    ));
                }
                Command::AddOffsetData {
                    src_offset,
                    dest_list,
                    ..
                } => todo!(),
                //{
                //    *count += 1;
                //    let mut total: u8 = 1;
                //    for dest in dest_list {
                //        total = total
                //            .wrapping_mul(tape[pointer.wrapping_add_signed(*dest.dst_offset)])
                //            .wrapping_mul(dest.multiplier);
                //    }
                //    tape[pointer.wrapping_add_signed(*src_offset)] =
                //        tape[pointer.wrapping_add_signed(*src_offset)].wrapping_add(*total);
                //}
                Command::SubOffsetData {
                    src_offset,
                    dest_list,
                    ..
                } => todo!(),
                Command::Output { id, .. } => {
                    append_io_syscall(out_string, 1, 1, *id, ".");
                }
                Command::Input { id, .. } => {
                    append_io_syscall(out_string, 0, 0, *id, ",");
                }
                Command::Loop { body, id, .. } => {
                    out_string.push_str("    # [\n");
                    out_string.push_str("    movb (%r12), %al\n");
                    out_string.push_str("    cmpb $0,     %al\n");
                    out_string.push_str(&format!("    je   loop{}_end\n", id));
                    out_string.push('\n');
                    out_string.push_str(&format!("loop{}:\n", id));

                    compile_rec(out_string, body);

                    out_string.push_str("     # ]\n");
                    out_string.push_str("    movb (%r12), %al\n");
                    out_string.push_str("    cmpb $0,     %al\n");
                    out_string.push_str(&format!("    jne  loop{}\n", id));
                    out_string.push('\n');
                    out_string.push_str(&format!("loop{}_end:\n", id));
                }
            }
        }
    }

    // Build assembly file
    let mut asm = String::new();
    append_assembly_header(&mut asm);
    compile_rec(&mut asm, commands);
    append_assembly_footer(&mut asm);

    if output_asm_file {
        let asm_filepath = replace_extension_filepath(src_filepath, ".s");
        let asm_filepath = strip_directories_filepath(&asm_filepath);
        let mut asm_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&asm_filepath)
            .expect("Unable to open output file");
        asm_file
            .write_all(asm.as_bytes())
            .expect("Unable to write to assembly file");
    }

    // Assemble and link output
    let object_filepath = replace_extension_filepath(src_filepath, ".o");
    let object_filepath = strip_directories_filepath(&object_filepath);
    if let Err(e) = assemble(&asm, &object_filepath) {
        eprintln!("{}", e);
        return;
    }
    if let Err(e) = link(&object_filepath, dest_filename, output_object_file) {
        eprintln!("{}", e);
    }
}

fn main() {
    let args = Args::parse();

    // Read the file contents
    let src_contents = match std::fs::read_to_string(&args.file_name) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading file {}: {}", args.file_name, e);
            std::process::exit(1);
        }
    };

    let mut commands = parser::parse(&src_contents);
    optimizer::optimize(&mut commands, args.optimization_level);
    if args.pretty_print {
        parser::pretty_print(&commands);
        return;
    }

    if args.interp || args.profile {
        interp(&mut commands);
        if args.profile {
            profiler::print_profile(&commands);
        }
        return;
    }

    compile(
        &commands,
        &args.file_name,
        &args.out_file,
        args.output_asm,
        args.output_object,
    );
}
