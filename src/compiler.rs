use crate::parser::{Command, Direction, OutputType};

const INIT_TAPE_SIZE: usize = 0x200000;
const INIT_POINTER_LOC: usize = 0x4000;

fn replace_extension_filepath(filepath: &str, ext: &str) -> String {
    return if let Some(pos) = filepath.rfind('.') {
        format!("{}{}", &filepath[..pos], ext)
    } else {
        format!("{}{}", &filepath, ext)
    };
}

fn strip_directories_filepath(filepath: &str) -> &str {
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
        use std::io::Write;
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
        .arg("-z")
        .arg("noexecstack")
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
.section .data

.align 32
mask_skip2:
  .byte 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF
mask_skip2_reverse:
  .byte 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00
mask_skip4:
  .byte 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF
mask_skip4_reverse:
  .byte 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00

.section .text

.globl main

main:
    pushq %rbp
    movq  %rsp, %rbp
    pushq {}
    pushq {}

    movq  ${}, %rdi
    call malloc

    pushq %rax            # Save tape address to the stack
    movq  %rax,      {} # Move tape address into callee saved register
    addq  ${}, {} # Move the pointer to the middle of the tape

    # Begin program code
"#,
        ptr_reg, full_byte_reg, INIT_TAPE_SIZE, ptr_reg, INIT_POINTER_LOC, ptr_reg
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

pub fn compile(
    commands: &[Command],
    src_filepath: &str,
    dest_filename: &str,
    output_asm_file: bool,
    output_object_file: bool,
) {
    use std::io::Write;

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

    fn compile_rec(out_string: &mut String, commands: &[Command], ptr_reg: &str, byte_reg: &str) {
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
                Command::Scan {
                    id,
                    direction,
                    skip_amount,
                    ..
                } => {
                    match skip_amount {
                        1 | 2 | 4 => match direction {
                            Direction::Right => {
                                out_string
                                    .push_str(&format!("    # [{}]\n", ">".repeat(*skip_amount)));
                                if *skip_amount == 2 {
                                    out_string.push_str("    vmovdqa mask_skip2(%rip), %ymm3\n")
                                } else if *skip_amount == 4 {
                                    out_string.push_str("    vmovdqa mask_skip4(%rip), %ymm3\n")
                                }
                                out_string.push_str("    xorq %rsi, %rsi\n");
                                out_string.push_str("    vpxor %ymm1, %ymm1, %ymm1\n");
                                out_string.push_str(&format!("vector{}_loop_start:\n", id));
                                out_string
                                    .push_str(&format!("    vmovdqu ({}, %rsi), %ymm0\n", ptr_reg));
                                if *skip_amount == 2 || *skip_amount == 4 {
                                    out_string.push_str("    vpor %ymm3, %ymm0, %ymm0\n");
                                }
                                out_string.push_str("    vpcmpeqb %ymm1, %ymm0, %ymm2\n");
                                out_string.push_str("    vpmovmskb %ymm2, %eax\n");
                                out_string.push_str("    testl %eax, %eax\n");
                                out_string.push_str(&format!("    jnz vector{}_found_zero\n", id));
                                out_string.push_str("    addq $32, %rsi\n");
                                out_string.push_str(&format!("    jmp vector{}_loop_start\n", id));
                                out_string.push_str(&format!("vector{}_found_zero:\n", id));
                                out_string.push_str("    bsfl %eax, %eax\n");
                                out_string.push_str("    addq %rax, %rsi\n");
                                out_string.push_str(&format!("    addq %rsi, {}\n", ptr_reg));
                                out_string.push('\n');
                            }
                            Direction::Left => {
                                out_string
                                    .push_str(&format!("    # [{}]\n", "<".repeat(*skip_amount)));
                                if *skip_amount == 2 {
                                    out_string
                                        .push_str("    vmovdqa mask_skip2_reverse(%rip), %ymm3\n")
                                } else if *skip_amount == 4 {
                                    out_string
                                        .push_str("    vmovdqa mask_skip4_reverse(%rip), %ymm3\n")
                                }
                                out_string.push_str("    movq $-31, %rsi\n");
                                out_string.push_str("    vpxor %ymm1, %ymm1, %ymm1\n");
                                out_string.push_str(&format!("vector{}_loop_start:\n", id));
                                out_string
                                    .push_str(&format!("    vmovdqu ({}, %rsi), %ymm0\n", ptr_reg));
                                if *skip_amount == 2 || *skip_amount == 4 {
                                    out_string.push_str("    vpor %ymm3, %ymm0, %ymm0\n");
                                }
                                out_string.push_str("    vpcmpeqb %ymm1, %ymm0, %ymm2\n");
                                out_string.push_str("    vpmovmskb %ymm2, %eax\n");
                                out_string.push_str("    testl %eax, %eax\n");
                                out_string.push_str(&format!("    jnz vector{}_found_zero\n", id));
                                out_string.push_str("    subq $32, %rsi\n");
                                out_string.push_str(&format!("    jmp vector{}_loop_start\n", id));
                                out_string.push_str(&format!("vector{}_found_zero:\n", id));
                                out_string.push_str("    bsrl %eax, %eax\n");
                                out_string.push_str("    addq %rax, %rsi\n");
                                out_string.push_str(&format!("    addq %rsi, {}\n", ptr_reg));
                                out_string.push('\n');
                            }
                        },
                        _ => {
                            // Run this as a normal loop
                            let loop_cmd;
                            match direction {
                                Direction::Right => {
                                    loop_cmd = Command::Loop {
                                        id: *id,
                                        body: vec![Command::IncPointer {
                                            amount: *skip_amount,
                                            count: 0,
                                        }],
                                        start_count: 0,
                                        end_count: 0,
                                    }
                                }
                                Direction::Left => {
                                    loop_cmd = Command::Loop {
                                        id: *id,
                                        body: vec![Command::DecPointer {
                                            amount: *skip_amount,
                                            count: 0,
                                        }],
                                        start_count: 0,
                                        end_count: 0,
                                    }
                                }
                            }
                            let new_cmds = vec![loop_cmd];
                            compile_rec(out_string, &new_cmds, ptr_reg, byte_reg)
                        }
                    }
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
                Command::Output { out_type, .. } => {
                    out_string.push_str("    # .\n");
                    match out_type {
                        OutputType::Const(val) => {
                            out_string.push_str(&format!("    movl ${}, %edi\n", val))
                        }
                        OutputType::Cell { offset } => out_string
                            .push_str(&format!("    movzbl {}({}), %edi\n", offset, ptr_reg)),
                    }
                    out_string.push_str("    call putchar\n");
                    out_string.push('\n');
                }
                Command::Input { offset, .. } => {
                    //append_io_syscall(out_string, 0, 0, *id, ptr_reg, ",");
                    out_string.push_str("    # ,\n");
                    out_string.push_str("    call getchar\n");
                    out_string.push_str(&format!("    movb %al, {}({})\n", offset, ptr_reg));
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
