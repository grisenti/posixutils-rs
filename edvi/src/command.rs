use std::str::FromStr;

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
    pub fn from_line(line: &str) -> Result<Command, String> {
        let mut parts = line.splitn(2, ' ');
        let cmd = match parts.next() {
            Some("a") => Command::Append(String::from(parts.next().unwrap_or(""))),
            Some("c") => Command::Change(String::from(parts.next().unwrap_or(""))),
            Some("d") => Command::Delete,
            Some("g") => {
                let mut g_parts = parts.next().unwrap().splitn(4, '/');
                let re = g_parts.next().unwrap().to_string();
                let replacement = g_parts.next().unwrap().to_string();
                let mut global = true;
                let mut count = false;
                let mut print = false;
                for flag in g_parts {
                    if flag == "g" {
                        global = true;
                    } else if flag == "n" {
                        count = true;
                    } else if flag == "p" {
                        print = true;
                    } else {
                        return Err(format!("Invalid flag: {}", flag));
                    }
                }
                Command::Global(re, replacement, global, count, print)
            }
            Some("G") => {
                let mut g_parts = parts.next().unwrap().splitn(2, '/');
                let re = g_parts.next().unwrap().to_string();
                let commands = match g_parts.next() {
                    Some(cmds) => parse_command_list(cmds)?,
                    None => vec![],
                };
                Command::InteractiveGlobalNotMatched(re, commands)
            }
            Some("i") => Command::Append(String::from(parts.next().unwrap_or(""))),
            Some("j") => Command::Move(1),
            Some("k") => Command::Move(-1),
            Some("m") => Command::Move(isize::from_str(parts.next().unwrap_or("")).unwrap()),
            Some("n") => Command::Print(PrintMode::NextLine),
            Some("p") => Command::Print(PrintMode::Line),
            Some("q") => Command::Quit,
            Some("r") => Command::Read(String::from(parts.next().unwrap_or(""))),
            Some("s") => {
                let mut s_parts = parts.next().unwrap().splitn(4, '/');
                let re = s_parts.next().unwrap().to_string();
                let replacement = s_parts.next().unwrap().to_string();
                Command::Global(re, replacement, false, false, false)
            }
            Some("t") => Command::Move(isize::from_str(parts.next().unwrap_or("")).unwrap()),
            Some("u") => Command::NoOp,
            Some("v") => {
                let mut v_parts = parts.next().unwrap().splitn(2, '/');
                let re = v_parts.next().unwrap().to_string();
                let commands = match v_parts.next() {
                    Some(cmds) => parse_command_list(cmds)?,
                    None => vec![],
                };
                Command::GlobalNotMatched(re, commands)
            }
            Some("w") => {
                let filename = parts.next().map(|s| s.to_string());
                Command::Write(filename)
            }
            Some("!") => {
                return Err(String::from("! command not supported"));
            }
            _ => return Err(format!("Invalid command: {}", line)),
        };
        Ok(cmd)
    }
}

fn parse_command_list(cmd_list: &str) -> Result<Vec<Command>, String> {
    let mut commands = Vec::new();
    for line in cmd_list.lines() {
        commands.push(Command::from_line(line)?)
    }
    Ok(commands)
}
