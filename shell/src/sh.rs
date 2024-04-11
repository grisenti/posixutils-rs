//
// Copyright (c) 2024 Jeff Garzik
//
// This file is part of the posixutils-rs project covered under
// the MIT License.  For the full license text, please see the LICENSE
// file in the root directory of this project.
// SPDX-License-Identifier: MIT
//

extern crate plib;

use gettextrs::{bind_textdomain_codeset, textdomain};
use plib::PROJECT_NAME;
use std::io::{self, Write};
use std::process;

enum Input {
    ChangeDir(String),
    Exec(String, Vec<String>),
    Exit,
    NoOp,
}

fn parse_input(rawline: &str) -> Input {
    let mut args: Vec<String> = rawline.split_whitespace().map(|s| s.to_string()).collect();
    if args.len() == 0 {
        return Input::NoOp;
    }
    let command = args.remove(0);

    match command.as_str() {
        "cd" => {
            if args.len() == 0 {
                Input::ChangeDir("".to_string())
            } else {
                Input::ChangeDir(args[0].clone())
            }
        }
        "exit" => Input::Exit,
        _ => {
            if args.len() == 0 {
                Input::Exec(command, Vec::new())
            } else {
                Input::Exec(command, args)
            }
        }
    }
}

fn exec_command(input: String, args: Vec<String>) -> io::Result<()> {
    let mut child = process::Command::new(input).args(args).spawn()?;

    child.wait()?;
    Ok(())
}

fn read_eval_print() -> io::Result<bool> {
    // display prompt
    print!("$ ");
    io::stdout().flush()?;

    // read a line of shell input
    let mut rawline = String::new();
    let n_read = io::stdin().read_line(&mut rawline)?;
    if n_read == 0 {
        return Ok(false);
    }

    // parse the input
    let input = parse_input(&rawline);

    // execute based on input
    match input {
        Input::ChangeDir(dir) => {
            if dir == "" {
                let home = std::env::var("HOME").unwrap();
                std::env::set_current_dir(home)?;
            } else {
                std::env::set_current_dir(dir)?;
            }
        }
        Input::Exit => return Ok(false),
        Input::Exec(input, args) => {
            exec_command(input, args)?;
        }
        Input::NoOp => {}
    }
    Ok(true)
}

fn read_eval_print_loop() -> io::Result<()> {
    loop {
        match read_eval_print() {
            Ok(false) => break,
            Err(e) => eprintln!("Error: {}", e),
            _ => {}
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    textdomain(PROJECT_NAME)?;
    bind_textdomain_codeset(PROJECT_NAME, "UTF-8")?;

    read_eval_print_loop()?;

    Ok(())
}
