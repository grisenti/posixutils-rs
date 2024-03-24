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
pub enum Command {
    Append(String),
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

impl Command {
    fn from_tokens(tokens: Vec<Token>) -> Result<Command, String> {
        match tokens[0] {
            Token::Command('q') => Ok(Command::Quit),
            Token::Command('Q') => Ok(Command::Quit),
            _ => unimplemented!(),
        }
    }

    pub fn from_line(line: &str) -> Result<Command, String> {
        match tokenizer(line) {
            Err(e) => Err(e.message),
            Ok(tokens) => Self::from_tokens(tokens),
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
