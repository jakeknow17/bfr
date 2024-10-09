mod compiler;
mod interp;
mod optimizer;
mod parser;
mod partial;
mod profiler;

use clap::Parser;

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

    /// Disables partial evaluation when compiling
    #[arg(long = "no-partial-eval")]
    disable_partial_eval: bool,
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
        interp::interp(&mut commands);
        if args.profile {
            profiler::print_profile(&commands);
        }
        return;
    }

    if !args.disable_partial_eval {
        partial::partial_eval(&mut commands);
    }

    compiler::compile(
        &commands,
        &args.file_name,
        &args.out_file,
        args.output_asm,
        args.output_object,
    );
}
