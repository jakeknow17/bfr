# bfr

**bfr** is a simple Brainfuck interpreter written in Rust. It reads a Brainfuck source file and executes it.

## Building

To build the project, you'll need [Rust](https://www.rust-lang.org/) and Cargo installed. Clone the repository and run the following command to build the interpreter:

```bash
cargo build --release
```

This will generate an optimized executable in the `target/release` directory.

## Usage

Once built, you can run `bfr` with several options. The basic usage is:

```bash
./target/release/bfr \[OPTIONS\] <FILE_NAME>
```
Where `<FILE_NAME>` is the path to the Brainfuck source file.

### Arguments:
- `<FILE_NAME>`: The Brainfuck source file to be processed.

### Options:
- `-p`, `--profile`: Enable profiling. This also enables interpretation of the source file.
- `-P`, `--pretty-print`: Pretty-print the parsed output. This disables interpretation and compilation.
- `-o <FILE>`, `--output <FILE>`: Specify the output file name. Defaults to `a.out` if not provided.
- `-i`, `--interp`: Interpret the Brainfuck source file without compiling it.
- `-S`, `--assembly`: Output the generated assembly file.
- `-c`, `--object`: Output an object file.
- `-O <LEVEL>`: Set the optimization level, where `<LEVEL>` is between 0 and 3. Default is 1.
- `-h`, `--help`: Show help information.
- `-V`, `--version`: Show the version information.

## Examples

To interpert a Brainfuck file:
```bash
./target/release/bfr path/to/your/program.bf
```

To compile Brainfuck file to an executable:
```bash
./target/release/bfr -o program.out path/to/your/program.bf
```

To output assembly code for a Brainfuck file:
```bash
./target/release/bfr -S path/to/your/program.bf
```

To pretty-print the parsed Brainfuck source (without execution):
```bash
./target/release/bfr --pretty-print path/to/your/program.bf
```
