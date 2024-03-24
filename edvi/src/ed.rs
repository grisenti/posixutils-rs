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

mod buffer;
mod command;

use buffer::{Buffer, Chunk};
use clap::Parser;
use command::Command;
use gettextrs::{bind_textdomain_codeset, textdomain};
use plib::PROJECT_NAME;
use std::fs;
use std::io::{self, BufRead, BufReader};

const MAX_CHUNK: usize = 1000000;

/// ed - edit text
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// Use string as the prompt string when in command mode.
    #[arg(short, long, default_value = "")]
    prompt: String,

    /// Suppress the writing of byte counts by e, E, r, and w commands
    #[arg(short, long)]
    silent: bool,

    /// If the file argument is given, ed shall simulate an e command on the file named by the pathname, file, before accepting commands from stdin
    file: Option<String>,
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

    fn read_file(&mut self, pathname: &str) -> io::Result<()> {
        let file = fs::File::open(pathname)?;
        let mut reader = BufReader::new(file);
        let mut cur_chunk = Chunk::new();

        loop {
            let mut line = String::new();
            let rc = reader.read_line(&mut line)?;
            if rc == 0 {
                break;
            }

            cur_chunk.push_line(&line);
            if cur_chunk.len() > MAX_CHUNK {
                self.buf.append(cur_chunk);
                cur_chunk = Chunk::new();
            }
        }

        if cur_chunk.len() > 0 {
            self.buf.append(cur_chunk);
        }

        self.buf.pathname = String::from(pathname);

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command line arguments
    let args = Args::parse();

    textdomain(PROJECT_NAME)?;
    bind_textdomain_codeset(PROJECT_NAME, "UTF-8")?;

    let mut ed = Editor::new();

    if let Some(pathname) = &args.file {
        if let Err(e) = ed.read_file(pathname) {
            eprintln!("{}: {}", pathname, e);
        }
    }

    loop {
        let mut input = String::new();

        if !args.prompt.is_empty() {
            print!("{}", args.prompt);
        }

        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("stdout: {}", e);
            std::process::exit(1);
        }

        if input.is_empty() {
            break;
        }

        println!("LINE={}", input.trim_end());

        if !ed.push_line(&input) {
            break;
        }
    }

    Ok(())
}
