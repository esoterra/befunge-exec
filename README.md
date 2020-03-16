# befunge-exec

befunge-exec is a simple command line application that acts as a Befunge interpretter and debugger.

## Background

Befunge 93 is an esoteric programming language where programs are 2 dimensional spaces of bytes.
They are stored ascii text files or equivalently utf-8 files which contain no larger than 1-byte values.
To execute the file a cursor moves through the space interpretting each byte as a command,
and then based on the command updating its position, direction, stack, and the space itself.
To learn more about Befunge 93 see [this page](https://esolangs.org/wiki/Befunge) on esolangs.org.

## Running befunge-exec

Currently befunge-exec can only be run in an interactive mode from the command line.

When it is run you will be prompted to name a file in the programs directory.
This name should not include the .b93 file extension.

Once this is done you will be prompted for commands.

### Commands

The format for each command is the value in the quoted string after the commands name.
When prompted for commands type a value matching a single command format and then hit enter.

* Step "s" - Executes one opcode
* Run "r" - Runs the program until a breakpoint is hit or the program terminates
* Breakpoint "b \<x\> \<y\>" - Places a breakpoint at the specified location
* Input "i.." - Sends the rest of your input into the input buffer of the interpreter byte for byte
* Position "p" - Prints the current (x,y) coordinates of the kernel with the top left corner as (0,0)
* Line "l" - Prints the current line of the program
* Debug "d" - Prints the Rust debug formatted value of the entire interpreter
* Quite "q" - Terminates the befunge-exec interactive debugger / interpreter
