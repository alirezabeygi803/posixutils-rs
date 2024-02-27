//
// Copyright (c) 2024 Jeff Garzik
//
// This file is part of the posixutils-rs project covered under
// the MIT License.  For the full license text, please see the LICENSE
// file in the root directory of this project.
// SPDX-License-Identifier: MIT
//

extern crate clap;
extern crate plib;

mod command;

use clap::Parser;
use command::Command;
use gettextrs::{bind_textdomain_codeset, textdomain};
use plib::PROJECT_NAME;
use std::fs;
use std::io::{self, BufRead, BufReader};

const MAX_CHUNK: usize = 1000000;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    pathname: String,
}

struct Chunk {
    data: String,

    first_line: u64,
    last_line: u64,
}

impl Chunk {
    fn new(line_no: u64) -> Chunk {
        Chunk {
            data: String::new(),
            first_line: line_no,
            last_line: line_no,
        }
    }

    fn from(s: &str) -> Chunk {
        Chunk {
            data: String::from(s),
            first_line: 0,
            last_line: 0,
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn push_line(&mut self, line: &str) {
        self.data.push_str(line);
        self.last_line = self.last_line + 1;
    }
}

struct Buffer {
    chunks: Vec<Chunk>,

    pathname: String,
    last_line: u64,
}

impl Buffer {
    fn new() -> Buffer {
        Buffer {
            chunks: Vec::new(),
            pathname: String::new(),
            last_line: 0,
        }
    }

    fn append(&mut self, chunk: Chunk) {
        self.last_line = chunk.last_line;
        self.chunks.push(chunk);
    }
}

struct Editor {
    in_cmd_mode: bool,
    buf: Buffer,

    cur_line: u64,

    inputs: Vec<String>,
}

impl Editor {
    fn new() -> Editor {
        Editor {
            in_cmd_mode: true,
            buf: Buffer::new(),
            cur_line: 0,
            inputs: Vec::new(),
        }
    }

    fn input_end(&mut self) -> bool {
        self.in_cmd_mode = true;

        // todo: flush to buffer...

        true
    }

    fn push_input_line(&mut self, line: &str) -> bool {
        if line == "." {
            self.input_end()
        } else {
            self.inputs.push(line.to_string());
            true
        }
    }

    fn push_cmd(&mut self, cmd: &Command) -> bool {
        println!("COMMAND: {:?}", cmd);

        let mut retval = true;
        match cmd {
            Command::Quit => {
                retval = false;
            }

            _ => {}
        }

        retval
    }

    fn push_cmd_line(&mut self, line: &str) -> bool {
        match Command::from_line(line) {
            Err(e) => {
                eprintln!("{}", e);
                true
            }
            Ok(cmd) => self.push_cmd(&cmd),
        }
    }

    fn push_line(&mut self, line: &str) -> bool {
        if self.in_cmd_mode {
            self.push_cmd_line(line.trim_end())
        } else {
            self.push_input_line(line)
        }
    }

    fn read_file(&mut self, pathname: &str) -> io::Result<()> {
        let file = fs::File::open(pathname)?;
        let mut reader = BufReader::new(file);
        let mut line_no = 1;
        let mut cur_chunk = Chunk::new(line_no);

        loop {
            let mut line = String::new();
            let rc = reader.read_line(&mut line)?;
            if rc == 0 {
                break;
            }

            line_no = line_no + 1;

            cur_chunk.push_line(&line);
            if cur_chunk.len() > MAX_CHUNK {
                self.buf.append(cur_chunk);
                cur_chunk = Chunk::new(line_no);
            }
        }

        if cur_chunk.len() > 0 {
            self.buf.append(cur_chunk);
        }

        self.buf.pathname = String::from(pathname);
        self.cur_line = line_no - 1;

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command line arguments
    let args = Args::parse();

    textdomain(PROJECT_NAME)?;
    bind_textdomain_codeset(PROJECT_NAME, "UTF-8")?;

    let mut state = Editor::new();

    match state.read_file(&args.pathname) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{}: {}", args.pathname, e);
        }
    }

    loop {
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_n) => {}
            Err(e) => {
                eprintln!("stdout: {}", e);
                std::process::exit(1);
            }
        }

        if input.is_empty() {
            break;
        }

        println!("LINE={}", input);

        if !state.push_line(&input) {
            break;
        }
    }

    Ok(())
}
