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
use command::{Address, AddressInfo, Command};
use gettextrs::{bind_textdomain_codeset, textdomain};
use plib::PROJECT_NAME;
use std::fs;
use std::io::{self, BufRead, BufReader};

const ERR_STR: &str = "?";

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

#[derive(Debug)]
enum EdOp {
    ReadInputLines,
    GotoLine(usize),
    Insert(bool),
}

#[derive(Debug)]
struct EdBundle {
    ops: Vec<EdOp>,
}

impl EdBundle {
    fn new() -> EdBundle {
        EdBundle { ops: Vec::new() }
    }

    fn push(&mut self, op: EdOp) {
        self.ops.push(op);
    }
}

#[derive(Debug)]
struct Editor {
    in_cmd_mode: bool,
    buf: Buffer,

    inputs: Vec<String>,

    exec_queue: Vec<EdBundle>,
}

impl Editor {
    fn new() -> Editor {
        Editor {
            in_cmd_mode: true,
            buf: Buffer::new(),
            inputs: Vec::new(),
            exec_queue: Vec::new(),
        }
    }

    fn bottom_half(&mut self) -> bool {
        while !self.exec_queue.is_empty() {
            let bundle = self.exec_queue.remove(0);
            for op in bundle.ops {
                match op {
                    EdOp::ReadInputLines => {
                        self.input_begin();
                    }

                    EdOp::GotoLine(line_no) => {
                        self.buf.set_cur_line(line_no);
                    }

                    EdOp::Insert(insert_before) => {
                        if self.inputs.len() == 0 {
                            continue;
                        }
                        let chunks = buffer::as_chunks(&self.inputs);
                        self.inputs.clear();
                        self.buf.insert(self.buf.cur_line, insert_before, &chunks);
                    }
                }
            }
        }

        true
    }

    fn input_begin(&mut self) {
        self.in_cmd_mode = false;
        assert_eq!(self.inputs.len(), 0);
    }

    fn input_end(&mut self) -> bool {
        self.in_cmd_mode = true;

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

    fn resolve_address(&self, addr_opt: &Option<Address>) -> Result<usize, String> {
        match addr_opt {
            None => Ok(self.buf.cur_line),
            Some(addr) => match addr.info {
                AddressInfo::Current => Ok(self.buf.cur_line),
                AddressInfo::Last => Ok(self.buf.last_line),
                AddressInfo::Line(line_no) => Ok(line_no),
                AddressInfo::Offset(offset) => {
                    let line_no = self.buf.cur_line as isize + offset;
                    if line_no < 0 {
                        Err("address out of range".to_string())
                    } else {
                        Ok(line_no as usize)
                    }
                }
                AddressInfo::Mark(_) => unimplemented!(),
                AddressInfo::RegexForward(_) => unimplemented!(),
                AddressInfo::RegexBack(_) => unimplemented!(),
            },
        }
    }

    fn push_cmd(&mut self, cmd: &Command) -> bool {
        println!("COMMAND: {:?}", cmd);

        let mut retval = true;
        match cmd {
            Command::Quit => {
                retval = false;
            }

            Command::Insert(addr_opt, mut insert_before) => {
                let mut bundle = EdBundle::new();
                bundle.push(EdOp::ReadInputLines);

                let line_no: usize = match self.resolve_address(addr_opt) {
                    Ok(line_no) => {
                        if insert_before && line_no == 0 {
                            1
                        } else if !insert_before && line_no == 0 {
                            insert_before = true;
                            1
                        } else {
                            line_no
                        }
                    }
                    Err(_) => {
                        println!("{}", ERR_STR);
                        return true;
                    }
                };
                bundle.push(EdOp::GotoLine(line_no));
                bundle.push(EdOp::Insert(insert_before));

                self.exec_queue.push(bundle);
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

            if (cur_chunk.len() + line.len()) > buffer::MAX_CHUNK {
                self.buf.append(cur_chunk);
                cur_chunk = Chunk::new();
            }
            cur_chunk.push_line(&line);
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

        if !ed.push_line(&input) || !ed.bottom_half() {
            break;
        }
    }

    Ok(())
}
