#!/usr/bin/env python3

import subprocess
import sys
import os

INPUT = b'\xb6\xba\x81%\xfe<W\x05\x84S"C\x138\x10\x8e\x1a\xec\xbe\xf7\xa1B\xd8T\xb6\xe7\xac\'\x0f\x7f\xd9ho\xe1Z\xac\xe3\xf1\x13\xa2\xedb\xa6\xed\x01\xeb\x16\xd5s\xf3\x97KC,\x15Y\x04\xcf\x9b-\xc8\xf2\xb1\x89oQV\xf4hXH\xe5\n\xa1l\xa6\xe8Lwvw\xc3\tM[Po\xdc\xf2\xb47su \x97\xa1\xf8r\xfc\x94n\xbbD\xb4\x18\xff\xa9c\xcd/\xe8\xd1\x7f[\xdf\xc4v2b\xaf\xd0\xbc\x08\xa3I\xfa\xfc5b\\/\r\xeb\x9d\x05\xe8\xcf\xc9h!\xe3\xa6:5\xf8\x99C\t\xc0yu%\x03\xcf8\xfd%\xcfs\x1aI\x93\xcaT\x17\xf9\xd5\xc8\xe3\x157s\x07\xa31\xbdI\xd5X\xbd\xd3q4a\xff\xc4\x9d\xcd\xc6\x99\x92e\x1b\t\x90\x08s\xdb\x1aN\'\x91d\x15v\xa3\xb4K\xaa\x9dv\xab\x90\x7fu\xa5L\xa7\xdb\x92\xf1\x01\x8er^\xf6-n\x04=\x19\x93W\xd0I\x8b\x0fp*\x81(\xcai\xb4\t\xa2\xf1\xfc\x8d\xe8\x8d<\xc5\xe1\x02\xfd\xae\x84q_r!\xba\x86G[\xcc/*;i\xc5\xfca\x820\x03-\xbe\xdan\xb6 \x14\xc6\xb8\xe7\x1f\x8e\xe0\x7f\x121\x80\xb6l\xc4\xc1\x16u5d\x16bj\x96\x17\x15W\x11\xb5v\xe6\xc9_\x98\xe3\xe7J\x8d\x8b\xeaG6\xeeY\x85\x88\\Od\x17\xabb\xce%\xcd0\xa0?\x99\xeb\x90\xce\ry\x1e\xc07\xf9\xdcm\x15o|\xfa\xc0g{\x9a\xebe\x8e1\xe4\xadC\x0e\xdb\xc4\xb1b\x04x\x90\x945KC\x14Y\xbc\x19\xc61A\xa0n\x9f\x133\xebv^\xf5\xff`C<\xb1 o\xd2\xf9s\'W1\xb8\xb6\xa9-Z\x7f=\x1a%\xddC\x1bM\x08\x0c\xfb\xc4?R\x8bR\xd6\xd2\x8a\x7f\x95\xc7\xbe\x9bk\xb2\x06\xbfc\xfd\xc2\xc2.8\x8e\xc1\x0e\xcb\x01\xf2\x91\xa3z\xb8\xabw\x13EPP\xc6\xe0\xff\x81\xed0\xd9,\xf7\x0bT\x07\xb0D\xab\x90X:\x00\xda\x7f\r\xbd\x97\xa3)\x03x\xc4RJ\x121)\xcb\x04\x95\x0e\xec*\x15\x83\xa0\x1d\xac~\xb9\x87\x1a,\xf6\xb2\x89)\xee\x82\xb2\xbd\xc5\x17\x02\xff\xb4\x18nXS\xfa\x9f\xb2\xeaj\xdf:\xc0\xcf>riE\x85\x94@\xc0\xc1\x17\xd0\xc8\xfbu\x02\xc4T\xde\xb3\x08\'Y\xb73"\x95\xec!\x0c\xdfHF0\x88\x8b\xc1\x93yr\x1f5\x1a\xa4+\xe0W&\xd1\xae\x01\xd9\x82\x17\x93\x0c\x85\xb4\x83\xc9\xa3\xf8\xc0\x1e\x9a\xe4~39&\x80\x057\xaeA\xc3\x8f\xe5\x0b\xdf\xc8\xe1\x00\x81O\xeaZKR\xea\xf3\xc0a\x97\xd6]JI\x9fIzY\x18\x00b\xecT:\xc9\xaf\xa4\x03\x16\x13@<t\x15c\xbc8J\x1e)\xe5(\xa2%\x1a\x9d\xa3\x9b\xa9Yh\xe3\x8f\x86"\x9a#\xa5\x14\xde\xbeb}R~N\xd3\xcb,\xdeSe\x9f\xc8\xa3\xdeW\x10\xe5b\xe0\xafH\x8e`\x7fD\xed<\x1dZ\xaa\xa0\x1d\x15&\xb5 \x0eIe!\xdbw,\xfd\x9f\xb1e\x11\x14,\x89!=\x9fn\xf2^\xb37|G.\xf6\xdd\xa8\xd7hn\xbe\x0b\xc4p}\x15\x8402\x04\xc6\x96/\xae\x1d\xca\x00EyK=I\x05K$-G\xff\x1au\xf7[\xa5\x01\xab\x0f\xc4#\xa8\xf5\xeb:L\n!\x01\xde\xef\xd888 \'x\xdc\x10I\xe3\x9d>CE\x0f\xed\x87\xe3&\x85\xba\xf5\xf2\xfe\xfa\x80\xe4*\xde\xcb\xc3k\xbb\x16\x85\xb1H\xc6\xba\xf4\xdf4/\xdf\xa0H\x867V+\xca\x08\x1e\xd5\xf1G$\x16\\\xf7[\xc2?V\xc6\xa3\x01\xdf\xf9\xf2\x10>\x15\xd8\xb4\x95x@0\x97\xe0\xd2\xd8n\xf3\xcc%\x00\xa4\x07v\no\xf8\xe0t\xd7{\x9d\xf9\x90\xf6\x9a\xae\x1eU=\x13V\x07-I\xb7\xe6\xea\xd4\xfdI\x03b\x82\x008\x84\xa4\xe1\x00\xe2\x84]k\x07\x91u%\x16\xc1E\x92\xf4\x95\xc4\xe7q\xc4\xadM\xab\xab\xd8\x8f\x0b{I\xc6]\tv\x93\x9a\x02\xbf\xfb\xaf\x11\xb1\x1c\xaew\x0c1\x82\xab\x879\r\x08\x17\x02Dn\xba-B\xfe\xcb\xa3\xd4)\xec\'Q\xec\xecd\x95\xb3\xb8J\xfc\xa0\x87c\x12\xbb\xbc\xf7\xe0\x9a\x0f\x9f\xa8\xf4n\xaf\xd7\xbfl\xa0+\xf6\xb8\xbd7'
COMPILER_PATH = "/home/knowl/Documents/projects/compilers/bfr/target/release/bfr"
TEST_FILE = "prog.b"

def run_interpreter(test_file, input):
    """
    Runs the Brainfuck interpreter with the specified test file and input.
    Returns the interpreter's output as bytes.
    """
    cmd = [COMPILER_PATH, '-i', test_file, '-O3']
    try:
        result = subprocess.run(
            cmd,
            input=INPUT,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,  # Raises CalledProcessError on non-zero exit
            timeout=5  # Kill the process if it runs for more than 5 seconds
        )
        return result.stdout
    except subprocess.TimeoutExpired:
        print(f"Interpreter timed out after 5 seconds.")
        sys.exit(1)
    except subprocess.CalledProcessError as e:
        print("Interpreter failed with the following error:")
        print(e.stderr.decode('utf-8', errors='replace'))  # Decode error messages as UTF-8
        sys.exit(1)

def run_compiler(test_file, input):
    """
    Compiles the Brainfuck code and runs the compiled binary with the specified input.
    Returns the compiler's output as bytes.
    """
    compile_cmd = [COMPILER_PATH, '-o', 'bf', test_file, '-O3']
    try:
        # Compile the Brainfuck code
        subprocess.run(
            compile_cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,  # Raises CalledProcessError on non-zero exit
            timeout=5  # Kill the process if it runs for more than 5 seconds
        )
    except subprocess.TimeoutExpired:
        print(f"Compiler timed out after 5 seconds.")
        sys.exit(1)
    except subprocess.CalledProcessError as e:
        print("Compiler failed with the following error:")
        print(e.stderr.decode('utf-8', errors='replace'))  # Decode error messages as UTF-8
        sys.exit(1)
    
    # Ensure the compiled binary exists and is executable
    if not os.path.isfile('./bf'):
        print("Compilation succeeded but the binary './bf' was not found.")
        sys.exit(1)
    if not os.access('./bf', os.X_OK):
        print("Compiled binary './bf' is not executable.")
        sys.exit(1)
    
    # Run the compiled binary
    run_cmd = ['./bf']
    try:
        run_result = subprocess.run(
            run_cmd,
            input=INPUT,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,  # Raises CalledProcessError on non-zero exit
            timeout=5  # Kill the process if it runs for more than 5 seconds
        )
        return run_result.stdout
    except subprocess.TimeoutExpired:
        print(f"Running the compiled binary timed out after 5 seconds.")
        sys.exit(1)
    except subprocess.CalledProcessError as e:
        print("Running the compiled binary failed with the following error:")
        print(e.stderr.decode('utf-8', errors='replace'))  # Decode error messages as UTF-8
        sys.exit(1)

def main():
    # Check if test file exists
    if not os.path.isfile(TEST_FILE):
        print(f"Test file '{TEST_FILE}' does not exist.")
        sys.exit(1)
    
    print("Running interpreter...")
    interpreter_output = run_interpreter(TEST_FILE, INPUT)
    
    print("Running compiler and executing compiled binary...")
    compiler_output = run_compiler(TEST_FILE, INPUT)
    
    # Compare the outputs as bytes
    if interpreter_output == compiler_output:
        print("Success: Outputs are identical.")
        sys.exit(1)
    else:
        print("Difference detected between interpreter and compiler outputs!")
        print("\n--- Interpreter Output (hex) ---")
        print(interpreter_output.hex())
        print("\n--- Compiler Output (hex) ---")
        print(compiler_output.hex())
        
        # Show a diff-like output for binary data
        import difflib
        diff = difflib.unified_diff(
            interpreter_output.hex().splitlines(),
            compiler_output.hex().splitlines(),
            fromfile='Interpreter Output',
            tofile='Compiler Output',
            lineterm=''
        )
        print("\n--- Differences ---")
        for line in diff:
            print(line)
        sys.exit(0)

if __name__ == '__main__':
    main()

