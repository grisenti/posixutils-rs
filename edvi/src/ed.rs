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
use std::io;

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
    fn new() -> Chunk {
        Chunk {
            data: String::new(),
            first_line: 0,
            last_line: 0,
        }
    }

    fn from(s: &str) -> Chunk {
        Chunk {
            data: String::from(s),
            first_line: 0,
            last_line: 0,
        }
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
}

struct Editor {
    in_cmd_mode: bool,
    buf: Buffer,

    inputs: Vec<String>,
}

impl Editor {
    fn new() -> Editor {
        Editor {
            in_cmd_mode: true,
            buf: Buffer::new(),
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command line arguments
    let args = Args::parse();

    textdomain(PROJECT_NAME)?;
    bind_textdomain_codeset(PROJECT_NAME, "UTF-8")?;

    let mut state = Editor::new();

    loop {
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(n) => {}
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
