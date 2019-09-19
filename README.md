# befunge-exec

befunge-exec is a simple command line application that acts as a Befunge interpretter and debugger.

## Commands

Once a program has been selected the application will offer a prompt where you can enter the following commands.
Each command is entered on its own line

* s - the step command, executes one opcode
* r - the run command, runs until a breakpoint is hit or the program terminates
* i.. - the input command, sends the rest of your input into the input buffer of the runtime byte for byte
* p - the position command, prints the current (x,y) coordinates of the kernel with the top left corner as (0,0)
* d - the debug command, prints the debug of the entire runtime
* b \<x\> \<y\> - the breakpoint command, adds a breakpoint at the location specified
* l - the line command, prints the current line of the program
* q - the quit command, terminates the runtime
