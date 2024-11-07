use crate::parser::{Command, Direction, OutputType};

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::targets::{InitializationConfig, Target, TargetMachine};
use inkwell::types::BasicType;
use inkwell::values::{FunctionValue, PointerValue};
use inkwell::{AddressSpace, OptimizationLevel};

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

fn get_data_ptr<'a>(
    context: &'a Context,
    builder: &'a Builder,
    tape_ptr: &'a PointerValue,
    offset: isize,
    func_name: &str,
) -> PointerValue<'a> {
    let ptr = builder
        .build_load(context.ptr_type(AddressSpace::default()), *tape_ptr, "ptr")
        .expect(&format!("Failed to create load for {func_name}"))
        .into_pointer_value();
    let data_ptr = if offset == 0 {
        ptr
    } else {
        unsafe {
            builder
                .build_gep(
                    context.i8_type(),
                    ptr,
                    &[context.i64_type().const_int(offset as u64, false)],
                    "out_offset",
                )
                .expect(&format!("Failed to create GEP for {func_name}"))
        }
    };
    data_ptr
}

pub fn compile(commands: &[Command], src_filepath: &str) {
    fn compile_rec(
        context: &Context,
        builder: &Builder,
        commands: &[Command],
        tape_ptr: &PointerValue,
        input_fn: &FunctionValue,
        output_fn: &FunctionValue,
    ) {
        for command in commands {
            let ptr_type = context.ptr_type(AddressSpace::default());
            let i32_type = context.i32_type();
            let i8_type = context.i8_type();
            match command {
                Command::IncPointer { amount, .. } => {
                    let ptr = builder
                        .build_load(ptr_type, *tape_ptr, "ptr")
                        .expect("Failed to create load for inc pointer")
                        .into_pointer_value();
                    let inc_ptr = unsafe {
                        builder
                            .build_gep(
                                i8_type,
                                ptr,
                                &[i32_type.const_int(*amount as u64, false)],
                                "inc_ptr",
                            )
                            .expect("Failed to create GEP for inc pointer")
                    };
                    builder
                        .build_store(*tape_ptr, inc_ptr)
                        .expect("Failed to create store for inc pointer");
                }
                Command::DecPointer { amount, .. } => {
                    let ptr = builder
                        .build_load(ptr_type, *tape_ptr, "ptr")
                        .expect("Failed to create load for dec pointer")
                        .into_pointer_value();
                    let dec_ptr = unsafe {
                        builder
                            .build_gep(
                                i8_type,
                                ptr,
                                &[i32_type.const_int(-(*amount as i64) as u64, false)],
                                "inc_ptr",
                            )
                            .expect("Failed to create GEP for dec pointer")
                    };
                    builder
                        .build_store(*tape_ptr, dec_ptr)
                        .expect("Failed to create store for dec pointer");
                }
                Command::IncData { offset, amount, .. } => {
                    let data_ptr = get_data_ptr(context, builder, tape_ptr, *offset, "inc data");
                    let val = builder
                        .build_load(i8_type, data_ptr, "val")
                        .expect("Failed to create load for value in inc data")
                        .into_int_value();
                    let inc = builder
                        .build_int_add(val, i8_type.const_int(*amount as u64, false), "inc")
                        .expect("Failed to build add in inc data");
                    builder
                        .build_store(data_ptr, inc)
                        .expect("Failed to create store for inc data");
                }
                Command::DecData { offset, amount, .. } => {
                    let data_ptr = get_data_ptr(context, builder, tape_ptr, *offset, "dec data");
                    let val = builder
                        .build_load(i8_type, data_ptr, "val")
                        .expect("Failed to create load for value in dec data")
                        .into_int_value();
                    let dec = builder
                        .build_int_sub(val, i8_type.const_int(*amount as u64, false), "dec")
                        .expect("Failed to build sub in sub data");
                    builder
                        .build_store(data_ptr, dec)
                        .expect("Failed to create store for dec data");
                }
                Command::SetData { offset, value, .. } => {
                    let data_ptr = get_data_ptr(context, builder, tape_ptr, *offset, "set data");
                    builder
                        .build_store(data_ptr, i8_type.const_int(*value as u64, false))
                        .expect("Failed to create store for set data");
                }
                Command::Scan {
                    id,
                    direction,
                    skip_amount,
                    ..
                } => {
                    let loop_cmd = match direction {
                        Direction::Right => Command::Loop {
                            id: *id,
                            body: vec![Command::IncPointer {
                                amount: *skip_amount,
                                count: 0,
                            }],
                            start_count: 0,
                            end_count: 0,
                        },
                        Direction::Left => Command::Loop {
                            id: *id,
                            body: vec![Command::DecPointer {
                                amount: *skip_amount,
                                count: 0,
                            }],
                            start_count: 0,
                            end_count: 0,
                        },
                    };
                    let new_cmds = vec![loop_cmd];
                    compile_rec(context, builder, &new_cmds, tape_ptr, input_fn, output_fn);
                }
                Command::AddOffsetData {
                    src_offset,
                    dest_offset,
                    multiplier,
                    inverted,
                    ..
                } => {
                    let src_data_ptr = get_data_ptr(
                        context,
                        builder,
                        tape_ptr,
                        *src_offset,
                        "add offset data src",
                    );
                    let dest_data_ptr = get_data_ptr(
                        context,
                        builder,
                        tape_ptr,
                        *dest_offset,
                        "add offset data dest",
                    );

                    let src_val = builder
                        .build_load(i8_type, src_data_ptr, "src_val")
                        .expect("Failed to load src val in add offset data")
                        .into_int_value();
                    let src_val = if *inverted {
                        builder
                            .build_int_sub(i8_type.const_zero(), src_val, "inv_src_val")
                            .expect("Failed to build sub in add offset data")
                    } else {
                        src_val
                    };
                    let src_val_32 = builder
                        .build_int_z_extend(src_val, i32_type, "src_val_32")
                        .expect("Failed to zero extend in add offset data");
                    let mult_src_val_32 = builder
                        .build_int_mul(
                            src_val_32,
                            i32_type.const_int(*multiplier as u64, false),
                            "mult_src_val_32",
                        )
                        .expect("Failed to build mult in add offset data");
                    let mult_src_val = builder
                        .build_int_truncate(mult_src_val_32, i8_type, "mult_src_val")
                        .expect("Failed to truncate in add offset data");

                    let dest_val = builder
                        .build_load(i8_type, dest_data_ptr, "dest_val")
                        .expect("Failed to load src val in add offset data")
                        .into_int_value();

                    let add_result = builder
                        .build_int_add(dest_val, mult_src_val, "add_result")
                        .expect("Failed to build add in add offset data");

                    builder
                        .build_store(dest_data_ptr, add_result)
                        .expect("Failed to build store in add offset data");
                }
                Command::SubOffsetData {
                    src_offset,
                    dest_offset,
                    multiplier,
                    inverted,
                    ..
                } => {
                    let src_data_ptr = get_data_ptr(
                        context,
                        builder,
                        tape_ptr,
                        *src_offset,
                        "sub offset data src",
                    );
                    let dest_data_ptr = get_data_ptr(
                        context,
                        builder,
                        tape_ptr,
                        *dest_offset,
                        "sub offset data dest",
                    );

                    let src_val = builder
                        .build_load(i8_type, src_data_ptr, "src_val")
                        .expect("Failed to load src val in sub offset data")
                        .into_int_value();
                    let src_val = if *inverted {
                        builder
                            .build_int_sub(i8_type.const_zero(), src_val, "inv_src_val")
                            .expect("Failed to build sub in sub offset data")
                    } else {
                        src_val
                    };

                    let src_val_32 = builder
                        .build_int_z_extend(src_val, i32_type, "src_val_32")
                        .expect("Failed to zero extend in sub offset data");
                    let mult_src_val_32 = builder
                        .build_int_mul(
                            src_val_32,
                            i32_type.const_int(*multiplier as u64, false),
                            "mult_src_val_32",
                        )
                        .expect("Failed to build mult in sub offset data");
                    let mult_src_val = builder
                        .build_int_truncate(mult_src_val_32, i8_type, "mult_src_val")
                        .expect("Failed to truncate in sub offset data");

                    let dest_val = builder
                        .build_load(i8_type, dest_data_ptr, "dest_val")
                        .expect("Failed to load src val in sub offset data")
                        .into_int_value();

                    let add_result = builder
                        .build_int_sub(dest_val, mult_src_val, "add_result")
                        .expect("Failed to build add in sub offset data");

                    builder
                        .build_store(dest_data_ptr, add_result)
                        .expect("Failed to build store in sub offset data");
                }
                Command::Output { out_type, .. } => {
                    let out_val = match out_type {
                        OutputType::Const(val) => i32_type.const_int(*val as u64, false),
                        OutputType::Cell { offset } => {
                            let data_ptr =
                                get_data_ptr(context, builder, tape_ptr, *offset, "output");

                            let out_val = builder
                                .build_load(i8_type, data_ptr, "out_val")
                                .expect("Failed to load tape val in output")
                                .into_int_value();
                            let out_val_32 = builder
                                .build_int_z_extend(out_val, i32_type, "out_val_zext")
                                .expect("Failed to zero-extend tape val in output");
                            out_val_32
                        }
                    };

                    builder
                        .build_call(*output_fn, &[out_val.into()], "output")
                        .expect("Failed to output value");
                }
                Command::Input { offset, .. } => {
                    let input = builder
                        .build_call(*input_fn, &[], "input")
                        .unwrap()
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                        .into_int_value();
                    let input_8 = builder
                        .build_int_truncate(input, i8_type, "input_8")
                        .expect("Failed to truncate input in input");

                    let data_ptr = get_data_ptr(context, builder, tape_ptr, *offset, "set data");
                    builder
                        .build_store(data_ptr, input_8)
                        .expect("Failed to create store for input");
                }
                Command::Loop { body, id, .. } => {
                    let parent_fn = builder.get_insert_block().unwrap().get_parent().unwrap();

                    let loop_cond_block =
                        context.append_basic_block(parent_fn, &format!("loop{}_cond", id));
                    let loop_body_block =
                        context.append_basic_block(parent_fn, &format!("loop{}_body", id));
                    let loop_end_block =
                        context.append_basic_block(parent_fn, &format!("loop{}_end", id));

                    builder
                        .build_unconditional_branch(loop_cond_block)
                        .expect("Failed to build first unconditional branch in loop");
                    builder.position_at_end(loop_cond_block);

                    let data_ptr = get_data_ptr(context, builder, tape_ptr, 0, "loop");
                    let val = builder
                        .build_load(i8_type, data_ptr, "val")
                        .expect("Failed to create load for value in loop")
                        .into_int_value();
                    let cmp = builder
                        .build_int_compare(
                            inkwell::IntPredicate::NE,
                            val,
                            i8_type.const_int(0, false),
                            "loop_cmp",
                        )
                        .expect("Failed to create cmp in loop");
                    builder
                        .build_conditional_branch(cmp, loop_body_block, loop_end_block)
                        .expect("Failed to build conditional branch in loop");
                    builder.position_at_end(loop_body_block);
                    compile_rec(context, builder, body, tape_ptr, input_fn, output_fn);
                    builder
                        .build_unconditional_branch(loop_cond_block)
                        .expect("Failed to build second unconditional branch in loop");
                    builder.position_at_end(loop_end_block);
                }
            }
        }
    }

    // Setup context
    let context = Context::create();
    let module = context.create_module("bf");
    let builder = context.create_builder();

    // Define types
    let i8_type = context.i8_type();
    let i32_type = context.i32_type();

    // Define main function
    let main_fn_type = i32_type.fn_type(&[], false);
    let main_fn = module.add_function("main", main_fn_type, None);
    let entry = context.append_basic_block(main_fn, "entry");
    builder.position_at_end(entry);

    // Create tape
    let tape_type = i8_type.array_type(INIT_TAPE_SIZE as u32);
    let tape = builder
        .build_malloc(tape_type, "tape")
        .expect("Failed to malloc tape");
    builder
        .build_memset(
            tape,
            1,
            i8_type.const_zero(),
            i32_type.const_int(INIT_TAPE_SIZE as u64, false),
        )
        .expect("Failed to memset the tape");

    // Setup tape pointer
    let init_ptr_offset = context.i64_type().const_int(INIT_POINTER_LOC as u64, false);
    let tape_ptr = unsafe {
        builder
            .build_gep(context.i8_type(), tape, &[init_ptr_offset], "tape_ptr")
            .expect("Failed to create GEP instruction")
    };
    let tape_ptr_alloca = builder
        .build_alloca(tape_ptr.get_type(), "tape_ptr_alloca")
        .expect("Failed to create tape pointer GEP instructio");
    builder
        .build_store(tape_ptr_alloca, tape_ptr)
        .expect("Failed to create tape pointer store");

    // Setup getchar
    let getchar_fn = module.get_function("getchar").unwrap_or_else(|| {
        let getchar_type = context.i32_type().fn_type(&[], false);
        module.add_function("getchar", getchar_type, None)
    });

    // Setup putchar
    let putchar_fn = module.get_function("putchar").unwrap_or_else(|| {
        let putchar_type = context
            .i32_type()
            .fn_type(&[context.i32_type().into()], false);
        module.add_function("putchar", putchar_type, None)
    });

    compile_rec(
        &context,
        &builder,
        &commands,
        &tape_ptr_alloca,
        &getchar_fn,
        &putchar_fn,
    );

    builder
        .build_return(Some(&context.i32_type().const_int(0, false)))
        .unwrap();

    // Optionally, verify the module
    if let Err(e) = module.verify() {
        eprintln!("Error verifying module: {}", e);
    }

    // TODO: Remove this print -----------------
    // println!("{}", module.print_to_string().to_string());
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
