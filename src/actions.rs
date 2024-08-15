use std::fmt::{Display, Write, Debug};

#[derive(Debug)]
pub enum Action {
    // prints `cd {0}` and changes the directory
    ChangeDir(Box<str>),
    // changes the directory
    ChangeDirQuiet(Box<str>),
    // prints `{0}` and runs it as a command
    RunCommand(Box<[Box<str>]>),
    // runs the command without showing its output
    RunCommandQuiet(Box<[Box<str>]>),
    // runs the command
    RunCommandOnlyOutput(Box<[Box<str>]>),
    // prints `edit {0}`, copies file from {2} to {1} and runs the editor on {1}
    RunEditor(Box<str>, Box<str>),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChangeDir(dir) => f.write_fmt(format_args!("cd {dir:?}")),
            Self::ChangeDirQuiet(dir) => f.write_fmt(format_args!("#cd {dir:?}")),
            Self::RunEditor(dst, src) => f.write_fmt(format_args!("+ {dst:?} {src:?}")),
            Self::RunCommand(args) => {
                for i in 0..args.len() {
                    if i != 0 {
                        f.write_char(' ')?;
                    }
                    Debug::fmt(&args[i], f)?;
                }
                Ok(())
            }
            Self::RunCommandQuiet(args) => {
                f.write_char('#')?;
                for i in 0..args.len() {
                    if i != 0 {
                        f.write_char(' ')?;
                    }
                    Debug::fmt(&args[i], f)?;
                }
                Ok(())
            }
            Self::RunCommandOnlyOutput(args) => {
                f.write_char('-')?;
                for i in 0..args.len() {
                    if i != 0 {
                        f.write_char(' ')?;
                    }
                    Debug::fmt(&args[i], f)?;
                }
                Ok(())
            }
        }
    }
}

pub fn parse_actions(contents: String) -> Vec<Action> {
    let mut actions = Vec::new();

    for line in contents.lines() {
        let cmd = if line.starts_with('#') || line.starts_with('-') || line.starts_with('+') { parse_command(&line[1..]) } else { parse_command(line) };
        if cmd.is_empty() {
            continue;
        }
        if cmd.len() == 2 && &*cmd[0] == "cd" && !line.starts_with('+') {
            if line.starts_with('-') || line.starts_with('#') {
                actions.push(Action::ChangeDirQuiet(cmd[1].clone()))
            } else {
                actions.push(Action::ChangeDir(cmd[1].clone()))
            }
            continue;
        }

        if line.starts_with('#') {
            actions.push(Action::RunCommandQuiet(cmd.into_boxed_slice()));
        } else if line.starts_with('-') {
            actions.push(Action::RunCommandOnlyOutput(cmd.into_boxed_slice()));
        } else if line.starts_with('+') {
            if cmd.len() != 2 {
                panic!("Editor Command `+` expected 2 arguments: A source file and a destination file");
            }
            actions.push(Action::RunEditor(cmd[0].clone(), cmd[1].clone()));
        } else {
            actions.push(Action::RunCommand(cmd.into_boxed_slice()));
        }
    }

    return actions;
}

fn parse_command(cmd: &str) -> Vec<Box<str>> {
    let mut command_list = vec![];
    let mut last = String::new();

    let mut escape = false;
    let mut in_str = false;

    for c in cmd.chars() {
        if escape {
            last.push(c);
        } else if c == '\\' {
            escape = true;
        } else if in_str {
            if c == '"' {
                in_str = false;
                command_list.push(std::mem::take(&mut last).into_boxed_str());
            } else {
                last.push(c);
            }
        } else if c == '"' {
            in_str = true;
        } else if c == ' ' {
            if !last.is_empty() {
                command_list.push(std::mem::take(&mut last).into_boxed_str());
            }
        } else {
            last.push(c);
        }
    }

    if in_str {
        last.insert(0, '"');
    }
    if escape {
        last.push('\\');
    }
    if !last.is_empty() {
        command_list.push(std::mem::take(&mut last).into_boxed_str());
    }

    command_list
}