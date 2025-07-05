# Asciinema Recording

Design for asciinema recording.

- Support `--cast` flag for debug command.
- Save recordings in `~/.bft/casts` if no name is specified.

Options:
1. Handle recording directly in `bft` by recording output in `Window` to an asciicast file.
2. Have `bft debug foobar.b93 --cast test1.cast` call `asciinema rec -c "bft debug foobar.b93" test1.cast` to let asciinema take care of it.

The second option is much simpler, but the first one would be cleaner and avoid any licensing weirdness since asciinema is GPLv3.

Which version of asciicast should I produce?
- ~~Version 1~~ (Fully deprecated)
- Version 2 - Being migrated away from for version 3 soon (tm)?
- Version 3 - The most recent, but it's not clear if it's fully supported everywhere yet.