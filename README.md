# befunge-exec

`befunge-exec` is a simple command line application that acts as a Befunge interpreter and debugger.

I may rename it in the near future ¯\\\_(ツ)\_/¯.

## Befunge

Befunge 93 is an esoteric programming language where programs are 2 dimensional spaces of bytes.
They are stored ascii text files or equivalently utf-8 files which contain no larger than 1-byte values.
To execute the file a cursor moves through the space interpreting each byte as a command
that affects the position & direction of the cursor, the stack, and the space itself.

To learn more about Befunge 93, check out the [Befunge page on esolangs.org](https://esolangs.org/wiki/Befunge).

The `befunge-exec` tooling faithfully supports the Befunge 93 spec, except the restriction that programs
must be 80x25 in size is relaxed. In the future, this may be updated to a new dialect
that supports Unicode Scalar Values instead of ASCII bytes, 32-bit values, and other features.

## Modes

`befunge-exec` has three modes: run, debug, and tui
- Run: 
- Debug: A simple command-line interpreter that provides basic operations.
- TUI: A work-in-progress Terminal User Interface (TUI) for visualizing and debugging befunge programs.

### Run

Executes the program, reads input from standard input, writes output to standard output, logs interpreter errors to standard error, and exits with status code 0 unless the interpreter encounters an error.

### Debug

Starts the debugger and provides a prompt for commands that allows the user to set breakpoints, step, run, etc. Input is provided by running the `i` command. Output of running/stepping programs is printed to standard output.

### TUI

> Note: The TUI is a WIP and does not yet support all of its intended features yet.

* The program state is displayed on the left-hand side and colored in a semantically-aware way.
* The current contents of the stack are visualized on the right-hand side.
* The bottom area contains tabs for console input/output, entering commands to affect the debugger execution (default tab), and a timeline tab that will eventually contain a log of commands and time-travel debugger feature.

![A terminal window with the title "befunge-exec: lessmore.b93" that is displaying the Terminal User Interface (TUI) of a debugger for the Befunge esoteric programming language. It is made up of Box Drawing Characters and styled with ANSI color codes. It has a main program area displaying a program for a simple guessing game that picks a random number that you try to guess,, a sidebar with an empty table titled Stack, and a set of tabs at the bottom called Console, Commands, and Timeline. The Commands tab is currently selected and shows an empty user input prompt and help output instructing the user on how to operate the debugger.](./docs/tui_example.png)

## Analysis Features

I intend for `befunge-exec` to embed powerful Befunge static and dynamic analysis features that enhance the user's ability to understand a given Befunge program and ensure that it does what it should or determine why it doesn't.

### Naive Path Analysis (Implemented)

This static analysis pass performs a breadth-first search, using a queue, of the cells reachable by the program assuming it does not modify itself in a way that changes cells that are visited/executed. This is what makes it possible to highlight characters that are executed as instructions differently from characters visited in quote mode and from those that aren't visited at all, and draw lines along the paths the cursor takes through empty space.

### Symbolic Evaluation (Not Implemented)

This static analysis pass would symbolically-execute the program
* Tracking the height of the stack and what is known about each cell, like if the value is known or known to be within a range.
* Literally executing deterministic instructions whose operands are known or tracking what is known about the output on the stack
* Forking when a `?` a `|`/`_` with input that can't be statically known to be zero or non-zero is encountered
* ...

### Time-travel Debugging (Not Implemented)

By keeping a log of each instruction that was executed and enough information to replay the execution forwards and also rewind it backwards, we would be able to step forwards/backwards and do things like step backwards from a breakpoint to see how it was reached.

### Assertions (Not Implemented)

This is more speculative, but the idea would be to define a syntax for specifying invariants that are statically or dynamically asserted to be true.

In dynamic checking, we would just ensure that the invariant isn't broken during execution.

For static checking, we would take advantage of symbolic evaluation to ensure that that it isn't possible for the invariant to be broken.
