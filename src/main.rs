// SPDX-License-Identifier: GPL-3.0-only

use std::{
    env, fs,
    io::{self, BufReader, BufWriter, Read, Write},
    path::Path,
    process::ExitCode,
};

struct Instruction {
    command: u8,
    argument: usize,
    offset: usize,
}

fn source_error(src: &[u8], offset: usize, msg: &str) -> io::Error {
    let prefix = &src[..offset];
    let line = prefix.iter().filter(|&&byte| byte == b'\n').count() + 1;
    let column = prefix
        .iter()
        .rposition(|&byte| byte == b'\n')
        .map_or(offset + 1, |index| offset - index);

    io::Error::new(io::ErrorKind::InvalidData, format!("{line}:{column}: {msg}"))
}

fn execute(src: &[u8], input: &mut impl Read, out: &mut impl Write) -> io::Result<()> {
    let mut code: Vec<Instruction> = Vec::new();
    let mut openings = Vec::new();

    for (offset, &command) in src.iter().enumerate() {
        if !b"><+-.,[]".contains(&command) {
            continue;
        }

        if b"><+-.".contains(&command) {
            if let Some(last) = code.last_mut() {
                if last.command == command && last.offset + last.argument == offset {
                    last.argument += 1;
                    continue;
                }
            }
        }

        let argument = match command {
            b'[' => {
                openings.push(code.len());
                0
            }
            b']' => {
                let opening = openings
                    .pop()
                    .ok_or_else(|| source_error(src, offset, "unmatched ']'"))?;
                if code.len() == opening + 2 {
                    let body = &code[opening + 1];
                    if b"+-".contains(&body.command) && body.argument & 1 == 1 {
                        // an odd step visits every byte val so this loop always reaches 0
                        code[opening].command = b'0';
                        code.truncate(opening + 1);
                        continue;
                    }
                    if b"><".contains(&body.command) {
                        let scan = Instruction {
                            command: if body.command == b'>' { b'R' } else { b'L' },
                            argument: body.argument,
                            offset: body.offset,
                        };

                        code[opening] = scan;
                        code.truncate(opening + 1);
                        continue;
                    }
                }
                code[opening].argument = code.len() + 1;
                opening + 1
            }
            _ => 1,
        };

        code.push(Instruction {
            command,
            argument,
            offset,
        });
    }

    if let Some(opening) = openings.pop() {
        return Err(source_error(src, code[opening].offset, "unmatched '['"));
    }

    let mut tape = [0u8; 30_000];
    let mut ptr = 0usize;
    let mut cursor = 0;

    while cursor < code.len() {
        let instruction = &code[cursor];

        match instruction.command {
            b'>' => {
                ptr = ptr
                    .checked_add(instruction.argument)
                    .filter(|&target| target < tape.len())
                    .ok_or_else(|| {
                        source_error(
                            src,
                            instruction.offset + tape.len() - ptr - 1,
                            "pointer past end of tape",
                        )
                    })?;
            }
            b'<' => {
                ptr = ptr
                    .checked_sub(instruction.argument)
                    .ok_or_else(|| source_error(src, instruction.offset + ptr, "pointer before start of tape"))?;
            }
            b'R' => {
                let stride = instruction.argument;

                if let Some(steps) = tape[ptr..].iter().step_by(stride).position(|&cell| cell == 0) {
                    ptr += steps * stride;
                } else {
                    return Err(source_error(
                        src,
                        instruction.offset + (tape.len() - ptr - 1),
                        "pointer past end of tape",
                    ));
                }
            }
            b'L' => {
                let stride = instruction.argument;

                if let Some(steps) = tape[..=ptr].iter().rev().step_by(stride).position(|&cell| cell == 0) {
                    ptr -= steps * stride;
                } else {
                    return Err(source_error(
                        src,
                        instruction.offset + ptr,
                        "pointer before start of tape",
                    ));
                }
            }
            b'+' => tape[ptr] = tape[ptr].wrapping_add(instruction.argument as u8),
            b'-' => tape[ptr] = tape[ptr].wrapping_sub(instruction.argument as u8),
            b'.' => {
                if instruction.argument == 1 {
                    out.write_all(&tape[ptr..ptr + 1])?;
                } else {
                    let bytes = [tape[ptr]; 1024];
                    let mut remaining = instruction.argument;

                    while remaining > 0 {
                        let count = remaining.min(bytes.len());

                        out.write_all(&bytes[..count])?;
                        remaining -= count;
                    }
                }
            }
            b',' => {
                out.flush()?;
                match input.read_exact(&mut tape[ptr..ptr + 1]) {
                    Ok(()) => {}
                    Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => tape[ptr] = 0,
                    Err(error) => return Err(error),
                }
            }
            b'[' if tape[ptr] == 0 => {
                cursor = instruction.argument;
                continue;
            }
            b']' if tape[ptr] != 0 => {
                cursor = instruction.argument;
                continue;
            }
            b'0' => tape[ptr] = 0,
            _ => {}
        }
        cursor += 1;
    }

    Ok(())
}

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let file = match (args.next(), args.next()) {
        (Some(file), None) => file,
        _ => {
            eprintln!("bf [file]");
            return ExitCode::FAILURE;
        }
    };

    let result = (|| {
        let source = fs::read(&file)?;
        let mut input = BufReader::new(io::stdin().lock());
        let mut output = BufWriter::new(io::stdout().lock());
        execute(&source, &mut input, &mut output)?;
        output.flush()
    })();

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bf: {}: {error}", Path::new(&file).display());
            ExitCode::FAILURE
        }
    }
}
