# Analysis Features

Befunge Tools contains static and dynamic analysis features that helps users understand Befunge programs and verify their behavior.

## Naive Path Analysis (Implemented)

This static analysis pass performs a breadth-first search, using a queue, of the cells reachable by the program assuming it does not modify cells that are reachable after modifying them.

This is what makes it possible to highlight characters that are executed as instructions differently from characters visited in quote mode and from those that aren't visited at all, and draw lines along the paths the cursor takes through empty space.

## Symbolic Evaluation (Not Implemented)

This static analysis pass would symbolically-execute the program
 - Tracking the height of the stack and what is known about each cell, like if the value is known or known to be within a range.
 - Literally executing deterministic instructions whose operands are known or tracking what is known about the output on the stack
 - Forking when a `?` a `|`/`_` with input that can't be statically known to be zero or non-zero is encountered
 - ...

## Time-travel Debugging (Not Implemented)

There is WIP to support [time-travel debugging](https://en.wikipedia.org/wiki/Time_travel_debugging) (similar to [record and replay](https://en.wikipedia.org/wiki/Record_and_replay_debugging)) using the `Record` trait and `Timeline` type. This will work by recording an event log that has enough information to be played forwards and backwards computing the program state.

The "Timeline" tab will show the sequence of instructions that have been executed and allow users to step forwards/backwards. This can be useful for seeing how a breakpoint was reached or what happened before pausing.

## Assertions (Not Implemented)

This is more speculative, but the idea would be to define a syntax for specifying invariants that are statically or dynamically asserted to be true.

In dynamic checking, we would just ensure that the invariant isn't broken during execution.

For static checking, we would take advantage of symbolic evaluation to ensure that that it isn't possible for the invariant to be broken.