# bfkit

a small brainfuck interpreter written in rust.

i wanted to make something that could run brainfuck without needing a bunch of dependencies or unnecessary features, so here it is.

## building

you'll need rust installed, obviously.

```sh
cargo build --release
```

the executable will be in `target/release/bf`.

## usage

```sh
bf hello.bf
```

that's it. give it a file and it'll execute it.

bfkit supports all 8 brainfuck instructions, uses a fixed tape of 30,000 8-bit cells, and reads/writes bytes directly through stdin and stdout.

there are a few optimizations too, mostly combining repeated instructions and avoiding having to look for matching brackets every time a loop runs. loops that only clear a cell are also optimized.

cells wrap around when they overflow or underflow, and attempting to move outside the tape will produce an error rather than silently wrapping the pointer around. malformed loops are rejected before execution, with line and column numbers to help figure out where you messed up.

EOF sets the current cell to zero.

## why?

i felt like writing a brainfuck interpreter.

that's literally it.

## license

licensed under [GPL-3.0-only](LICENSE). the original template's MIT notice is
retained in [LICENSE-MIT](LICENSE-MIT).
