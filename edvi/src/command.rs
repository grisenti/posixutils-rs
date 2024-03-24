//
// Copyright (c) 2024 Jeff Garzik
//
// This file is part of the posixutils-rs project covered under
// the MIT License.  For the full license text, please see the LICENSE
// file in the root directory of this project.
// SPDX-License-Identifier: MIT
//

use std::iter;

#[derive(Debug)]
struct SyntaxError {
    message: String,
}

enum Token {
    CurrentLine,
    LastLine,
    Number(u64),
    Mark(char),
    RegexForward(String),
    RegexBack(String),
    Offset(isize),

    AddressSeparator(char),
    Command(char),
    EOF,
}

impl SyntaxError {
    fn new(message: String) -> Self {
        SyntaxError { message }
    }
}

fn tokenizer(input: &str) -> Result<Vec<Token>, SyntaxError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(ch) = iter.next() {
        match ch {
            ch if ch.is_whitespace() => continue,
            '.' => tokens.push(Token::CurrentLine),
            '$' => tokens.push(Token::LastLine),
            'a'..='z' | 'A'..='Z' => {
                tokens.push(Token::Command(ch));
            }
            '1'..='9' => {
                let n: u64 = iter::once(ch)
                    .chain(iter::from_fn(|| {
                        iter.by_ref().next_if(|s| s.is_ascii_digit())
                    }))
                    .collect::<String>()
                    .parse()
                    .unwrap();

                tokens.push(Token::Number(n));
            }
            '\'' => match iter.next() {
                None => return Err(SyntaxError::new(String::from("missing mark char"))),
                Some(mark_ch) => {
                    if mark_ch.is_ascii_alphabetic() {
                        tokens.push(Token::Mark(mark_ch));
                    } else {
                        return Err(SyntaxError::new(format!(
                            "unrecognized mark character {}",
                            ch
                        )));
                    }
                }
            },
            ',' | ';' => {
                tokens.push(Token::AddressSeparator(ch));
            }
            '/' | '?' => {
                let mut bre = String::new();
                let mut escaped = false;
                loop {
                    let bre_ch_res = iter.next();
                    if bre_ch_res.is_none() {
                        return Err(SyntaxError::new(String::from("unterminated regex")));
                    }

                    let bre_ch = bre_ch_res.unwrap();
                    match bre_ch {
                        // regex terminator
                        '/' | '?' => {
                            if escaped {
                                bre.push(bre_ch);
                            } else {
                                if ch == '/' {
                                    tokens.push(Token::RegexForward(bre));
                                } else {
                                    tokens.push(Token::RegexBack(bre));
                                }
                                break;
                            }
                        }

                        // escape char
                        '\\' => {
                            if escaped {
                                bre.push(bre_ch);
                            } else {
                                escaped = true;
                            }
                        }

                        // everything else
                        _ => bre.push(bre_ch),
                    }
                }
            }

            '+' | '-' => {
                let n: isize = iter::once(ch)
                    .chain(iter::from_fn(|| {
                        iter.by_ref().next_if(|s| s.is_ascii_digit())
                    }))
                    .collect::<String>()
                    .parse()
                    .unwrap();

                tokens.push(Token::Offset(n));
            }
            _ => return Err(SyntaxError::new(format!("unrecognized character {}", ch))),
        }
    }

    tokens.push(Token::EOF);
    Ok(tokens)
}

#[derive(Debug)]
pub enum AddressInfo {
    Current,
    Last,
    Line(u64),
    Mark(char),
    RegexForward(String),
    RegexBack(String),
    Offset(isize),
}

#[derive(Debug)]
pub struct Address {
    addr: AddressInfo,
    offsets: Vec<isize>,
}

impl Address {
    fn new() -> Self {
        Address {
            addr: AddressInfo::Current,
            offsets: Vec::new(),
        }
    }

    fn add_offset(&mut self, offset: isize) {
        self.offsets.push(offset);
    }
}

#[derive(Debug)]
pub enum Command {
    Append(Option<Address>),
    Change(String),
    Copy(usize),
    Delete,
    Global(String, String, bool, bool, bool),
    GlobalNotMatched(String, Vec<Command>),
    InteractiveGlobalNotMatched(String, Vec<Command>),
    Move(isize),
    NoOp,
    Print(PrintMode),
    Read(String),
    Quit,
    Write(Option<String>),
}

#[derive(Debug)]
pub enum PrintMode {
    Line,
    NextLine,
    PreviousLine,
}

enum ParseState {
    Address,
    SepOffCommand,
    Command,
}

impl Command {
    fn parse(mut tokens: Vec<Token>) -> Result<Command, String> {
        let mut addr = Address::new();
        let mut addr_dirty = false;
        let mut addrvec = Vec::new();
        let mut state = ParseState::Address;

        while tokens.len() > 0 {
            let mut done_with_token = false;
            let token = tokens.remove(0);
            match state {
                // todo: handle separator indicating implicit addressing
                ParseState::Address => {
                    state = ParseState::SepOffCommand;
                    addr_dirty = true;
                    match token {
                        Token::CurrentLine => addr.addr = AddressInfo::Current,
                        Token::LastLine => addr.addr = AddressInfo::Last,
                        Token::Number(u64) => addr.addr = AddressInfo::Line(u64),
                        Token::Mark(char) => addr.addr = AddressInfo::Mark(char),
                        Token::RegexForward(String) => {
                            addr.addr = AddressInfo::RegexForward(String)
                        }
                        Token::RegexBack(String) => addr.addr = AddressInfo::RegexBack(String),
                        Token::Offset(isize) => addr.addr = AddressInfo::Offset(isize),
                        Token::Command(_) => {
                            tokens.insert(0, token);
                            state = ParseState::Command;
                            continue;
                        }
                        _ => return Err(String::from("unexpected token")),
                    }
                }

                ParseState::SepOffCommand => {
                    if addr_dirty {
                        addrvec.push(addr);
                        addr = Address::new();
                        addr_dirty = false;
                    }
                    match token {
                        Token::AddressSeparator(',') => {
                            state = ParseState::Address;
                        }
                        Token::AddressSeparator(':') => {
                            state = ParseState::Address;
                        }
                        Token::Offset(isize) => {
                            addr.add_offset(isize);
                        }
                        Token::Command(_) => {
                            tokens.insert(0, token);
                            state = ParseState::Command;
                            continue;
                        }
                        _ => return Err(String::from("unexpected token")),
                    }
                }

                ParseState::Command => match token {
                    Token::Command('a') => {
                        if addrvec.len() > 1 {
                            return Err("append command takes at most one address".to_string());
                        }
                        return Ok(Command::Append(addrvec.pop()));
                    }
                    Token::Command('q') => {
                        if addrvec.len() > 0 {
                            return Err("quit command takes no address".to_string());
                        }
                        return Ok(Command::Quit);
                    }
                    Token::Command(_) => {
                        return Err("unrecognized command".to_string());
                    }
                    _ => return Err(String::from("unexpected token")),
                },
            }
        }

        Err("address-and-command parse error".to_string())
    }

    pub fn from_line(line: &str) -> Result<Command, String> {
        match tokenizer(line) {
            Err(e) => Err(e.message),
            Ok(tokens) => Self::parse(tokens),
        }
    }
}

fn parse_command_list(cmd_list: &str) -> Result<Vec<Command>, String> {
    let mut commands = Vec::new();
    for line in cmd_list.lines() {
        commands.push(Command::from_line(line)?)
    }
    Ok(commands)
}
