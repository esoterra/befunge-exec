<div align="center">
  <h1>Befunge Tools - <code>bft</code></h1>

  <p>
    <strong>A collection of command line tools for executing, analyzing, and visualizing Befunge code.</strong>
  </p>
</div>

## Befunge

[Befunge 93](https://esolangs.org/wiki/Befunge) is an esoteric programming language where
* **programs are 2-dimensional grids** of `u8` cells, not sequences of lines;
* **the program counter is 2-dimensional** and can move up, down, left, or right!
* **instructions are ASCII characters** and occupy a single cell in the grid;
* **instructions pop/push** from a stack of `i32` cells;
* **programs can modify themselves** to store data and change program flow.

Befunge Tools currently targets Befunge 93 except that programs may be larger than 80x25. In the future, this may be updated to a new dialect with a Befunge 93 compatibility mode.

## Befunge Tools

Befunge Tools is a collection of command line tools for executing, analyzing, and visualizing Befunge code.

- [X] the `run` command which is a no-frills Befunge interpreter.
  - [ ] (Planned) support use in shebang interpreter directive
- [X] the `debug` command which launches an interactive TUI environment.
  - [X] command tab with debugger run/step/pause functionality
  - [X] console tab with interactive virtual terminal
  - [ ] (Planned) timeline tab with time-travel debugging
  - [X] program visualization with path-aware highlighting
  - [X] stack visualization sidebar
  - [X] breakpoint support

## Run

> Execute `bft run ./path/to/file.b93` in your terminal.

Runs the program and
* reads input from standard input,
* writes output to standard output,
* logs interpreter errors to standard error,
* exits with status code 0 unless the interpreter encounters an error.

## Debug - TUI Debugger

> Execute `bft debug ./path/to/file.b93 2> log.txt` in your terminal.

(Logging will probably become configurable or automatically placed in files in the future, but for now it is printed to standard error by default for development purposes and redirecting it is recommended/expected)

Launches the interactive debugger Terminal User Interface (TUI) with the specified program loaded in.

<figure>
  <a href="https://asciinema.org/a/jePiNWuNsG1lP4csxoTmJMhHT"><img src="https://asciinema.org/a/jePiNWuNsG1lP4csxoTmJMhHT.png" alt="A terminal window displaying the Terminal User Interface (TUI) of a debugger for the Befunge esoteric programming language. It is made up of Box Drawing Characters and styled with ANSI color codes. It has a main program area displaying a program for a simple guessing game that picks a random number that you try to guess,, a sidebar with an empty table titled Stack, and a set of tabs at the bottom called Console, Commands, and Timeline."></a>
  <figcaption align="center">The interactive debugger UI. Click it to see a demo at asciinema.org!</figcaption>
</figure>

### Logging

The debugger automatically saves logs to `~/.bft/logs`. The log level is controlled by the `--log-level` argument.