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

    /// Output .s assembly file
    #[arg(short = 'S', long = "assembly")]
    output_asm: bool,
}

pub fn interp(commands: &mut [parser::Command]) {
    use parser::Command;

    fn interp_rec(commands: &mut [parser::Command], tape: &mut [u8], pointer: &mut usize, pc: &mut usize) {
        while *pc < commands.len() {
            match &mut commands[*pc] {
                Command::IncPointer { amount, ref mut count } => {
                    *count += 1;
                    *pointer += *amount;
                },
                Command::DecPointer { amount, ref mut count } => {
                    *count += 1;
                    *pointer -= *amount;
                },
                Command::IncData { offset, amount, ref mut count } => {
                    *count += 1;
                    tape[pointer.saturating_add_signed(*offset)] = tape[pointer.saturating_add_signed(*offset)].wrapping_add(*amount);
                },
                Command::DecData { offset, amount, ref mut count } => {
                    *count += 1;
                    tape[pointer.saturating_add_signed(*offset)] = tape[pointer.saturating_add_signed(*offset)].wrapping_sub(*amount);
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
        &filepath[pos+1..]
    } else {
        filepath
    };
}

fn assemble_and_link_string(asm_string: &str, src_filepath: &str, dest_file: &str) {
    let object_filepath = replace_extension_filepath(src_filepath, ".o");
    let object_filepath = strip_directories_filepath(&object_filepath);

    // Step 1: Spawn the `as` process, capturing its stdout
    let mut as_process = std::process::Command::new("as")
        .arg("-o")
        .arg(&object_filepath)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn 'as' process");

    // Write the assembly code to the stdin of the `as` process
    {
        let stdin = as_process.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(asm_string.as_bytes()).expect("Failed to write to stdin");
    }

    // Wait for `as` to finish
    let as_status = as_process
        .wait()
        .expect("Failed to wait on 'as' process");

    if !as_status.success() {
        eprintln!("`as` failed with exit code: {:?}", as_status.code());
        return;
    }

    // Step 2: Spawn the `ld` process, using `as_output.stdout` as its input
    let mut ld_process = std::process::Command::new("ld")
        .arg("-o")
        .arg(dest_file)        // The output binary to be linked
        .arg(object_filepath)
        .spawn()
        .expect("Failed to spawn 'ld' process");

    // Wait for `ld` to finish linking
    let ld_status = ld_process
        .wait()
        .expect("Failed to wait on 'ld' process");

    if !ld_status.success() {
        eprintln!("`ld` failed with code: {:?}", ld_status.code());
    }
}

fn append_assembly_header(out_string: &mut String) {
    out_string.push_str(r#"
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
    pushq %r12
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

    # Begin program code"#);
}

fn append_assembly_footer(out_string: &mut String) {
    out_string.push_str(r#"
    # Should be at the bottom of main
    
    # Unmap memory from mmap
    movq $11,       %rax # Munmap system call number
    movq %r12,      %rdi # Address
    movq $0x200000, %rsi # 2 MB size
    syscall

    # TODO: Check if munmap failed

    movq %rbp, %rsp
    popq %r12
    popq %rbp

    # Return to _start
    ret
"#);
}

fn compile(commands: &[parser::Command], src_filename: &str, dest_filename: &str, output_asm_file: bool) {
    use parser::Command;

    fn compile_rec(out_string: &mut String, commands: &[parser::Command]) {
        for command in commands {
            match command {
                Command::IncPointer { amount, .. } => {
                    if *amount == 1 {
                        out_string.push_str("    incq %r12            # >\n");
                    } else {
                        out_string.push_str(&format!("{:24}# >{}\n", format!("    addq ${}, %r12", amount), amount));
                    }
                },
                Command::DecPointer { amount, .. } => {
                    if *amount == 1 {
                        out_string.push_str("    decq %r12            # <\n");
                    } else {
                        out_string.push_str(&format!("{:24} # <{}\n", format!("    subq ${}, %r12", amount), amount));
                    }
                },
                Command::IncData { offset, amount, .. } => {
                    let offset_str = if *offset == 0 { String::from("") } else { offset.to_string() };
                    if *amount == 1 {
                        out_string.push_str(&format!("{:24} # +\n", format!("    movb {}(%r12), %al", &offset_str)));
                        out_string.push_str("    incb %al\n");
                    } else {
                        out_string.push_str(&format!("{:24} # +{}\n", format!("    movb {}(%r12), %al", &offset_str), amount));
                        out_string.push_str(&format!("    addb ${}, %al\n", amount));
                    }
                    out_string.push_str(&format!("    movb %al, {}(%r12)\n", &offset_str));
                },
                Command::DecData { offset, amount, .. } => {
                    let offset_str = if *offset == 0 { String::from("") } else { offset.to_string() };
                    if *amount == 1 {
                        out_string.push_str(&format!("{:24} # +\n",format!("    movb {}(%r12), %al", &offset_str)));
                        out_string.push_str("    decb %al\n");
                    } else {
                        out_string.push_str(&format!("{:24} # +{}\n", format!("    movb {}(%r12), %al", &offset_str), amount));
                        out_string.push_str(&format!("    subb ${}, %al\n", amount));
                    }
                    out_string.push_str(&format!("    movb %al, {}(%r12)\n", &offset_str));
                },
                Command::Output { .. } => {
                    out_string.push_str("    movq $1,   %rax      # .\n");
                    out_string.push_str("    movq $1,   %rdi\n");
                    out_string.push_str("    movq %r12, %rsi\n");
                    out_string.push_str("    movq $1,   %rdx\n");
                    out_string.push_str("    syscall\n");
                }
                Command::Input { .. } => {
                    out_string.push_str("    movq $0,   %rax      # ,\n");
                    out_string.push_str("    movq $0,   %rdi\n");
                    out_string.push_str("    movq %r12, %rsi\n");
                    out_string.push_str("    movq $1,   %rdx\n");
                    out_string.push_str("    syscall\n");
                },
                Command::Loop { body, id, .. } => {
                    out_string.push_str("    movb (%r12), %al     # [\n");
                    out_string.push_str("    cmpb $0,     %al\n");
                    out_string.push_str(&format!("    je   loop{}_end\n", id));
                    out_string.push_str(&format!("loop{}:\n", id));

                    compile_rec(out_string, body);

                    out_string.push_str("    movb (%r12), %al     # ]\n");
                    out_string.push_str("    cmpb $0,     %al\n");
                    out_string.push_str(&format!("    jne  loop{}\n", id));
                    out_string.push_str(&format!("loop{}_end:\n", id));
                }
            }
        }
    }

    let mut asm = String::new();
    append_assembly_header(&mut asm);
    compile_rec(&mut asm, commands);
    append_assembly_footer(&mut asm);

    if output_asm_file {
        println!("Outputting asm!");
        let asm_filepath = replace_extension_filepath(src_filename, ".s");
        let asm_filepath = strip_directories_filepath(&asm_filepath);
        let mut asm_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&asm_filepath)
            .expect("Unable to open output file");
        asm_file.write_all(asm.as_bytes()).expect("Unable to write to assembly file");
    }

    assemble_and_link_string(&asm, src_filename, dest_filename);
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
    optimizer::optimize(&mut commands);
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

    compile(&commands, &args.file_name, &args.out_file, args.output_asm);
}
