use crate::parser::{Command, Direction, OutputType};

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target, TargetMachine};
use inkwell::types::BasicType;
use inkwell::OptimizationLevel;

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

fn clang(asm_file: &str, dest_file: &str, keep_object: bool) -> Result<(), String> {
    let mut clang_cmd = std::process::Command::new("clang");

    clang_cmd.arg("-o").arg(dest_file).arg(asm_file);

    if keep_object {
        clang_cmd.arg("-c");
    }

    let mut clang_process = clang_cmd.spawn().map_err(|_| {
        "Error: `clang` compiler not found. Please ensure it is installed on your system."
            .to_string()
    })?;

    let clang_status = clang_process
        .wait()
        .map_err(|_| "Error: Failed to wait for clang process.".to_string())?;

    if !clang_status.success() {
        return Err(format!(
            "Error: Clang failed with exit code: {:?}",
            clang_status.code()
        ));
    }

    Ok(())
}

pub fn compile(
    commands: &[Command],
    src_filepath: &str,
    dest_filename: &str,
    output_binary_file: bool,
    output_object_file: bool,
) {
    fn compile_rec(commands: &[Command]) {
        for command in commands {
            match command {
                Command::IncPointer { amount, .. } => {
                    todo!();
                }
                Command::DecPointer { amount, .. } => {
                    todo!();
                }
                Command::IncData { offset, amount, .. } => {
                    todo!();
                }
                Command::DecData { offset, amount, .. } => {
                    todo!();
                }
                Command::SetData { offset, value, .. } => {
                    todo!();
                }
                Command::Scan {
                    id,
                    direction,
                    skip_amount,
                    ..
                } => {
                    todo!();
                }
                Command::AddOffsetData {
                    src_offset,
                    dest_offset,
                    multiplier,
                    inverted,
                    ..
                } => {
                    todo!();
                }
                Command::SubOffsetData {
                    src_offset,
                    dest_offset,
                    multiplier,
                    inverted,
                    ..
                } => {
                    todo!();
                }
                Command::Output { out_type, .. } => match out_type {
                    OutputType::Const(val) => {
                        todo!();
                    }
                    OutputType::Cell { offset } => {
                        todo!();
                    }
                },
                Command::Input { offset, .. } => {
                    todo!();
                }
                Command::Loop { body, id, .. } => {
                    todo!();
                }
            }
        }
    }

    // Setup context
    let context = Context::create();
    let module = context.create_module("bf");
    let builder = context.create_builder();

    // Define main function
    let i32_type = context.i32_type();
    let main_fn_type = i32_type.fn_type(&[], false);
    let main_fn = module.add_function("main", main_fn_type, None);
    let entry = context.append_basic_block(main_fn, "entry");
    builder.position_at_end(entry);

    // Setup calloc
    let calloc_fn = module.get_function("calloc").unwrap_or_else(|| {
        let calloc_type = context.ptr_type(inkwell::AddressSpace::default()).fn_type(
            &[context.i64_type().into(), context.i64_type().into()],
            false,
        );
        module.add_function("calloc", calloc_type, None)
    });

    // Setup printf
    let printf_fn = module.get_function("printf").unwrap_or_else(|| {
        let printf_type = context.i32_type().fn_type(
            &[context.ptr_type(inkwell::AddressSpace::default()).into()],
            true,
        );
        module.add_function("printf", printf_type, None)
    });

    // Allocate memory for the tape
    let tape_size = context.i64_type().const_int(INIT_TAPE_SIZE as u64, false);
    let cell_size = context.i64_type().const_int(1, false);
    let tape = builder
        .build_call(
            calloc_fn,
            &[tape_size.into(), cell_size.into()],
            "tape_start",
        )
        .unwrap()
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_pointer_value();

    // TODO: Remove this printf call -----------
    // Create the format string "%d\n" as a global string
    let format_str = builder
        .build_global_string_ptr("%p\n", "format_str")
        .expect("Can't create global format");

    // Call printf with the format string and the integer value
    builder
        .build_call(
            printf_fn,
            &[format_str.as_pointer_value().into(), tape.into()],
            "printf_call",
        )
        .unwrap();
    // -----------------------------------------

    // Setup tape pointer
    let init_ptr_offset = context.i64_type().const_int(INIT_POINTER_LOC as u64, false);
    let tape_ptr = unsafe {
        builder
            .build_gep(context.i8_type(), tape, &[init_ptr_offset], "tape_ptr")
            .expect("Failed to create GEP instruction")
    };

    builder
        .build_return(Some(&context.i32_type().const_int(7, false)))
        .unwrap();

    // compile_rec(commands);

    // Optionally, verify the module
    if let Err(e) = module.verify() {
        eprintln!("Error verifying module: {}", e);
    }

    // TODO: Remove this print -----------------
    println!("{}", module.print_to_string().to_string());
    // -----------------------------------------

    // Initialize the native target
    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");

    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).expect("Could not get target from triple");
    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::Aggressive,
            inkwell::targets::RelocMode::Default,
            inkwell::targets::CodeModel::Default,
        )
        .expect("Could not create target machine");

    // Emit object code
    let object_filepath = replace_extension_filepath(src_filepath, ".o");
    let object_filepath = strip_directories_filepath(&object_filepath);
    target_machine
        .write_to_file(
            &module,
            inkwell::targets::FileType::Object,
            object_filepath.as_ref(),
        )
        .expect("Failed to write object file");
}
