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

    Ok(tokens)
}

#[derive(Debug)]
pub enum AddressInfo {
    Current,
    Last,
    Line(usize),
    Mark(char),
    RegexForward(String),
    RegexBack(String),
    Offset(isize),
}

#[derive(Debug)]
pub struct Address {
    pub info: AddressInfo,
    pub offsets: Vec<isize>,
}

impl Address {
    fn new() -> Self {
        Address {
            info: AddressInfo::Current,
            offsets: Vec::new(),
        }
    }

    fn add_offset(&mut self, offset: isize) {
        self.offsets.push(offset);
    }
}

#[derive(Debug)]
pub enum Command {
    Insert(Option<Address>, bool),
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
    Write(Option<Address>, Option<Address>, Option<String>, bool),
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
            let token = tokens.remove(0);
            match state {
                // todo: handle separator indicating implicit addressing
                ParseState::Address => {
                    state = ParseState::SepOffCommand;
                    addr_dirty = true;
                    match token {
                        Token::CurrentLine => addr.info = AddressInfo::Current,
                        Token::LastLine => addr.info = AddressInfo::Last,
                        Token::Number(v) => addr.info = AddressInfo::Line(v as usize),
                        Token::Mark(ch) => addr.info = AddressInfo::Mark(ch),
                        Token::RegexForward(s) => {
                            addr.info = AddressInfo::RegexForward(s);
                        }
                        Token::RegexBack(s) => addr.info = AddressInfo::RegexBack(s),
                        Token::Offset(i) => addr.info = AddressInfo::Offset(i),
                        Token::AddressSeparator(',') => {
                            addr.info = AddressInfo::Line(1);
                            state = ParseState::SepOffCommand;
                        }
                        Token::AddressSeparator(':') => {
                            state = ParseState::SepOffCommand;
                        }
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
                            return Err("Append command takes at most one address".to_string());
                        }
                        return Ok(Command::Insert(addrvec.pop(), false));
                    }
                    Token::Command('i') => {
                        if addrvec.len() > 1 {
                            return Err("Insert command takes at most one address".to_string());
                        }
                        return Ok(Command::Insert(addrvec.pop(), true));
                    }
                    Token::Command('q') => {
                        if addrvec.len() > 0 {
                            return Err("quit command takes no address".to_string());
                        }
                        return Ok(Command::Quit);
                    }
                    Token::Command('w') => {
                        if addrvec.len() > 2 {
                            return Err("write command takes at most two addresses".to_string());
                        }
                        return Ok(Command::Write(addrvec.pop(), addrvec.pop(), None, false));
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
