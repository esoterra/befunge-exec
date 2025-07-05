# TUI Design

The TUI does not use an existing TUI framework or design system. The exact layout, borders, and interactions were all designed and worked out by hand for this project.

## Layout

The design of the TUI uses box-drawing characters to create a frame
that separates the screen into 5 areas

```
╔══════════════════╦═══════════════╗
║                  ║               ║
║   Program View   ║     Stack     ║
║                  ║               ║
╟──────────────────╨───────────────╢
║          Tab Headings            ║
╠══════════════════╦═══════════════╣
║                  ║    Sidebar    ║
║   Tab Contents   ║    Corner     ║
║                  ║               ║
╚══════════════════╩═══════════════╝
```

## Dimensions

The sizes of the areas (excluding borders) is given using the following table.

| Area | Width | Height |
|-|-|-|
| Tab Headings | width | 1 cell |
| Sidebar Corner | 15 cells | 7 cells |
| Tab Contents | width - 3 (borders) - 15 (sidebar) cells | 7 cells |
| Stack | 15 cells | height - 4 (borders) - 1 (headings) - 7 (corner) | 
| Program View | width - 3 (borders) - 15 (sidebar) cells | height - 4 (borders) - 1 (headings) - 7 (corner) |

The bottom area has 7 cells height because that worked out nicely for fitting the Tab Contents views and also is enough room to give the cursor X and Y a place to be on the sidebar corner and room for a cute logo.

The sidebar is 15 cells wide because that is the amount of space needed to have 11 cells for showing the `i32` stack values, 3 cells for the represented character and 1 cell for the border between.

All other dimensions are sized to make use of the remaining available space.

## Minimum size

TODO: Explain minimum size for the main design

TODO: Design mini-scale design for when the window is smaller

## Design Files

You can find files with various design pieces in the `./docs/designs` folder.

- Tabs
  - [Console](./designs/tab_console.txt)
  - [Command](./designs/tab_command.txt)
  - [Timeline](./designs/tab_timeline.txt) - [Brainstorming](./designs/timeline.md)
- Sidebar
  - [Corner](./designs/sidebar_corner.txt)
  - [Stack Collapsing](./designs/stack_collapsing.md)
- [Resizing](./designs/resizing.md)
- [Original Concept](./designs/original_concept.txt)
- [Index of Glyphs](./designs/index_of_glyphs.md) (outdated)

## Acknowledgements

* The initial design was all done in text editors
* The logo is pulled from the [ASCII Art Archive](https://www.asciiart.eu/animals/cats)
* Revisions to the design have been designed using the [ASCII Draw Studio](https://www.asciiart.eu/ascii-draw-studio/app)