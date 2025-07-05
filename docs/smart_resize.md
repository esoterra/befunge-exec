# Smart Resize

Some WIP code I decided not to finish implementing that would only redraw the parts of the screen that need to be when resizing the window.

This ended up being fairly complicated and not clearly worth it.

```rs
/// From the perspective of a given screen update,
/// there are 6 main areas that may or may not need to be drawn
/// not counting the frame as its own piece.
///
/// If a resize has occurred, the portion of the program view
/// that was present before and isn't covered now remains unaffected.
/// If the width/height has shrunk, portions of the previous programming
/// area will need to be drawn over. Spaces will need to be actually drawn
/// instead of skipped since there may be something to cover up.
/// If the width/height has grown, portions of the UI that previously did not exist
/// (i.e. would have ben off screen) or contained tab, stack, or corner elements
/// will need to be drawn over.
///
/// Whenever the tab area needs to be completely redrawn, like if the window
/// resizes or someone switches tabs, we can clear from the top of the
/// tab section down. This means that frame drawing can still skip empty spaces.
///
/// Whenever the stack area needs to be completely redrawn, we will just
/// draw it over what was there including spaces to clear out previous contents.
///
/// ╔══════════════╤═══════════════════╦═══════╗
/// ║ Program View ┆ Right Resize Area ║ Stack ║
/// ╟╌╌╌╌╌╌╌╌╌╌╌╌╌╌┴╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╢       ║
/// ║ Lower Resize Area                ║       ║
/// ╟──────────────────────────────────╫───────╢
/// ║ Tabs Area                        ║Befunge║ <- corner info/logo
/// ╚══════════════════════════════════╩═══════╝
///
///
fn update(&mut self, window: &mut Window) -> io::Result<()> {
    window.start_frame()?;

    if let ResizeState::ResizedFrom { cols: old_width, rows: old_height } = self.resize {
        let new_width = window.width();
        let new_height = window.height();
        let right_resize = old_width < new_width;
        let lower_resize = old_height < new_height;
        
        let Dimensions { cols: old_cols, rows: old_rows } = ProgramView::dimensions_for_size(old_width, old_height);
        let Dimensions { cols: new_cols, rows: new_rows } = ProgramView::dimensions(window);
        
        if right_resize {
            let x = (old_cols as u8)..(1+new_cols as u8);
            let y = 0..(1+old_rows as u8);
            ProgramDisplay {
                analysis: &self.analysis,
                interpreter: &self.interpreter,
                x,
                y,
                fill: true,
            }.draw(window)?;
        }

        // TODO: draw upper stack area
        
        if lower_resize {
            window.clear_down(old_rows+1)?;
            ProgramDisplay {
                analysis: &self.analysis,
                interpreter: &self.interpreter,
                x: 0..(1+new_cols as u8),
                y: (old_rows as u8)..(1+new_rows as u8),
                fill: false,
            }.draw(window)?;
            // and lower stack area
        } else {
            let Dimensions { cols: _, rows } = ProgramView::dimensions(&window);
            window.clear_down(rows)?;
        }

        // TODO: draw tabs and corner areas

    }

    self.resize = ResizeState::Clean;

    // TODO: draw debugger cursor

    window.end_frame()
}
```