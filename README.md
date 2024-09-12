# bfr

**bfr** is a simple Brainfuck interpreter written in Rust. It reads a Brainfuck source file and executes it.

## Building

To build the project, you'll need [Rust](https://www.rust-lang.org/) and Cargo installed. Clone the repository and run the following command to build the interpreter:

```bash
cargo build --release
```

This will generate an optimized executable in the `target/release` directory.

## Usage

Once built, you can run the interpreter by providing the path to a Brainfuck source file as a command-line argument:

```bash
./target/release/bfr path/to/your/program.bf
```
