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
                Command::Output { ref mut count } => {
                    *count += 1;
                    let buf = vec![tape[*pointer]];
                    std::io::stdout().write_all(&buf)?;
                }
                Command::Input { ref mut count } => {
                    use std::io::Read;

                    *count += 1;
                    let mut input_buf: [u8; 1] = [0; 1];
                    if let Err(..) = std::io::stdin().read_exact(&mut input_buf) {
                        tape[*pointer] = 255; // -1
                    } else {
                        tape[*pointer] = input_buf[0];
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
        .arg("-dynamic-linker")
        .arg("/lib64/ld-linux-x86-64.so.2")
        .arg("/usr/lib/x86_64-linux-gnu/crt1.o")
        .arg("/usr/lib/x86_64-linux-gnu/crti.o")
        .arg("-lc")
        .arg(object_filepath)
        .arg("/usr/lib/x86_64-linux-gnu/crtn.o")
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

fn append_assembly_header(out_string: &mut String, ptr_reg: &str, full_byte_reg: &str) {
    out_string.push_str(&format!(
        r#"
.section .text

.globl main

main:
    pushq %rbp
    movq  %rsp, %rbp
    pushq {}
    pushq {}

    movq  $0x200000, %rdi
    call malloc

    pushq %rax            # Save tape address to the stack
    movq  %rax,      {} # Move tape address into callee saved register
    addq  $0x100000, {} # Move the pointer to the middle of the tape

    # Begin program code
"#,
        ptr_reg, full_byte_reg, ptr_reg, ptr_reg
    ));
}

fn append_assembly_footer(out_string: &mut String, ptr_reg: &str, full_byte_reg: &str) {
    out_string.push_str(&format!(
        r#"    # At bottom of main
    
    popq %rax
    popq {}
    popq {}
    popq %rbp

    # Return to _start
    ret
"#,
        full_byte_reg, ptr_reg
    ));
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
        amount: usize,
        single_op: &str,
        multi_op: &str,
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
        reg: &str,
        byte_reg: &str,
        comment: &str,
    ) {
        let offset_str = if offset == 0 {
            String::from("")
        } else {
            format!("{}", offset)
        };
        out_string.push_str(&format!("    # {}\n", comment));
        out_string.push_str(&format!("    movb {}({}), {}\n", offset_str, reg, byte_reg));
        if amount == 1 {
            out_string.push_str(&format!("    {} {}\n", single_op, byte_reg));
        } else {
            out_string.push_str(&format!("    {} ${}, {}\n", multi_op, amount, byte_reg));
        }
        out_string.push_str(&format!("    movb {}, {}({})\n", byte_reg, offset_str, reg));
        out_string.push('\n');
    }

    fn compile_rec(
        out_string: &mut String,
        commands: &[parser::Command],
        ptr_reg: &str,
        byte_reg: &str,
    ) {
        for command in commands {
            match command {
                Command::IncPointer { amount, .. } => {
                    append_pointer_op(out_string, *amount, "incq", "addq", ptr_reg, ">");
                }
                Command::DecPointer { amount, .. } => {
                    append_pointer_op(out_string, *amount, "decq", "subq", ptr_reg, "<");
                }
                Command::IncData { offset, amount, .. } => {
                    append_data_op(
                        out_string, *offset, *amount, "incb", "addb", ptr_reg, byte_reg, "+",
                    );
                }
                Command::DecData { offset, amount, .. } => {
                    append_data_op(
                        out_string, *offset, *amount, "decb", "subb", ptr_reg, byte_reg, "-",
                    );
                }
                Command::SetData { offset, value, .. } => {
                    let offset_str = if *offset == 0 {
                        String::from("")
                    } else {
                        format!("{}", offset)
                    };
                    out_string.push_str(&format!(
                        "{:24} # ={}\n",
                        format!("    movb ${}, {}({})", value, offset_str, ptr_reg),
                        value
                    ));
                }
                Command::AddOffsetData {
                    src_offset,
                    dest_offset,
                    multiplier,
                    inverted,
                    ..
                } => {
                    out_string.push_str(&format!(
                        "    movb {}({}), {}\n",
                        src_offset, ptr_reg, byte_reg
                    ));
                    if *inverted {
                        out_string.push_str(&format!("    notb {}\n", byte_reg));
                        out_string.push_str(&format!("    incb {}\n", byte_reg));
                    }
                    out_string.push_str(&format!("    movb ${}, %al\n", multiplier));
                    out_string.push_str(&format!("    mulb {}\n", byte_reg));
                    out_string.push_str(&format!(
                        "    movb {}({}), {}\n",
                        dest_offset, ptr_reg, byte_reg
                    ));
                    out_string.push_str(&format!("    addb %al, {}\n", byte_reg));
                    out_string.push_str(&format!(
                        "    movb {}, {}({})\n",
                        byte_reg, dest_offset, ptr_reg
                    ));
                }

                //*count += 1;
                //let mut src_val = if *inverted {
                //    0u8.wrapping_sub(tape[pointer.wrapping_add_signed(*src_offset)]) as usize
                //} else {
                //    tape[pointer.wrapping_add_signed(*src_offset)] as usize
                //};
                //src_val = src_val.wrapping_mul(*multiplier) % 256;
                //tape[pointer.wrapping_add_signed(*dest_offset)] =
                //    tape[pointer.wrapping_add_signed(*dest_offset)].wrapping_add(src_val as u8);
                Command::SubOffsetData {
                    src_offset,
                    dest_offset,
                    multiplier,
                    inverted,
                    ..
                } => {
                    out_string.push_str(&format!(
                        "    movb {}({}), {}\n",
                        src_offset, ptr_reg, byte_reg
                    ));
                    if *inverted {
                        out_string.push_str(&format!("    notb {}\n", byte_reg));
                        out_string.push_str(&format!("    incb {}\n", byte_reg));
                    }
                    out_string.push_str(&format!("    movb ${}, %al\n", multiplier));
                    out_string.push_str(&format!("    mulb {}\n", byte_reg));
                    out_string.push_str(&format!(
                        "    movb {}({}), {}\n",
                        dest_offset, ptr_reg, byte_reg
                    ));
                    out_string.push_str(&format!("    subb %al, {}\n", byte_reg));
                    out_string.push_str(&format!(
                        "    movb {}, {}({})\n",
                        byte_reg, dest_offset, ptr_reg
                    ));
                }
                Command::Output { .. } => {
                    //append_io_syscall(out_string, 1, 1, *id, ptr_reg, ".");
                    out_string.push_str("    # .\n");
                    out_string.push_str(&format!("    movzbl ({}), %edi\n", ptr_reg));
                    out_string.push_str("    call putchar\n");
                    out_string.push('\n');
                }
                Command::Input { .. } => {
                    //append_io_syscall(out_string, 0, 0, *id, ptr_reg, ",");
                    out_string.push_str("    # ,\n");
                    out_string.push_str("    call getchar\n");
                    out_string.push_str(&format!("    movb %al, ({})\n", ptr_reg));
                    out_string.push('\n');
                }
                Command::Loop { body, id, .. } => {
                    out_string.push_str("    # [\n");
                    out_string.push_str(&format!("loop{}:\n", id));
                    out_string.push_str(&format!("    movb ({}), {}\n", ptr_reg, byte_reg));
                    out_string.push_str(&format!("    cmpb $0,     {}\n", byte_reg));
                    out_string.push_str(&format!("    je   loop{}_end\n", id));
                    out_string.push('\n');

                    compile_rec(out_string, body, ptr_reg, byte_reg);

                    out_string.push_str("     # ]\n");
                    out_string.push_str(&format!("    jmp  loop{}\n", id));
                    out_string.push('\n');
                    out_string.push_str(&format!("loop{}_end:\n", id));
                }
            }
        }
    }

    let ptr_reg = "%r12";
    let full_byte_reg = "%r13";
    let byte_reg = "%r13b";

    // Build assembly file
    let mut asm = String::new();
    append_assembly_header(&mut asm, ptr_reg, full_byte_reg);
    compile_rec(&mut asm, commands, ptr_reg, byte_reg);
    append_assembly_footer(&mut asm, ptr_reg, full_byte_reg);

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
