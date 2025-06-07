# Language Extension Ideas

Brainstorming for a new Befunge dialect.

## Goals

1. **Make a "pleasant" Funge** - Make the language something I would want to write in more and think others might want to too.
2. **Keep it simple** - Make the language easy to implement and learn, but hard to master and often understand. Necessarily, stay closer to Befunge-93 than Funge-98 did.
3. **Add new elements that "fit"** - Add elements from Funge-98 and entirely new elements that "fit" into Befunge's style and emphasize its iconic elements.

## Iconic Elements

Exactly which elements are "iconic" is debatable, but in my mind they are

1. **Programs as grids** with toroidal topology
2. **Having a 2-Dimensional Program Counter** with "direction" and focus on relative motion
3. **Single "character" context-free instructions**
4. **Stack-orientation** as the basis of data flow
5. **Self-modification** to change program flow and store data

## Features

### 32-bit and Unicode

Befunge-93 only has 8-bit values, allows programs to be a maximum of 80x25 in size, and uses Ascii.

My ideal funge would use 32-bit cell sizes everywhere, have a `u32::MAX` size limited playground, and have strong Unicode support based on representing programs in UTF-8 and using Unicode Scalar Values in place of Ascii characters.

> **Note about Funge 98:**
> Funge 98 allows for choosing 32-bit values and rendering them as unicode but it doesn't require it. I would require it.
> Leaving things up to implementors is great for flexibility and generalizing over the space of funges that Funge 98 did, but I don't think it's the best for implementors and program authors.

### `s` Set / Dump Instruction

An instruction that pops a value from the stack, writes it to the next cell in front of the cursor, and then jumps over it.
* I think it would be cute for it to dump the top value of its stack and hop over it
* It's essentially just a combination of the behaviors of `#` and `p`

This instruction scores a perfect 5/5 fitting with each of the key themes.

### Slices and Stack Frames

Instead of going to a full stack stack that lets you swap and interact with both stacks
* adding stack "frames" that allow for some sub-procedure isolation and structuring and operations.
* operating on length-topped stacks of data on the stack.
* Instead of adding C-like null terminated `0"gnirtS"`, adding length-prefixed `"gnirtS"n` (ga-nirts-en).
* Adding operations that operate on length-topped stacks of data

#### Operations

* **Swap**
  * `cba3` `fed3` -> `fed3` `cba3`
* **Concatenate**
  * `fed3` `cba3` -> `fedcba6`
* **Element-wise arithmetic**
  * Addition: `7523` `132` -> `7653`

### Inclusions from Funge-98

* The skip instruction `;` that jumps to the next one of itself
  * e.g. `12;34;+` -> `3`

# References

https://esoteric.codes/blog/befunge
https://catseye.tc/view/The-Dossier/article/Befunge%20Silver%20Jubilee%20Retrospective.md
https://git.catseye.tc/Befunge-93/blob/master/eg/anagram.bf