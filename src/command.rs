use std::{io::Write, process::{Command, Stdio}};

pub fn run_command(cmd: &[Box<str>]) -> std::io::Result<()> {
    if cmd.len() < 1 {
        return Ok(());
    }
    let mut args = cmd.iter().map(|el| &**el);
    let Some(cmd) = args.next() else { return Ok(()); };
    let mut cmd = Command::new(cmd);
    cmd.args(args);
    if let Ok(dir) = std::env::current_dir() {
        cmd.current_dir(dir);
    }
    _ = std::io::stderr().flush();
    _ = std::io::stdout().flush();
    // todo: clear stdin?

    cmd.stderr(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stdin(Stdio::inherit());
    // todo: clear stdin?

    let mut child = cmd.spawn()?;
    while child.try_wait()?.is_none() {}

    Ok(())
}

pub fn run_command_quiet(cmd: &[Box<str>]) -> std::io::Result<()> {
    if cmd.len() < 1 {
        return Ok(());
    }
    let mut args = cmd.iter().map(|el| &**el);
    let Some(cmd) = args.next() else { return Ok(()); };
    let mut cmd = Command::new(cmd);
    cmd.args(args);
    if let Ok(dir) = std::env::current_dir() {
        cmd.current_dir(dir);
    }
    _ = std::io::stderr().flush();
    _ = std::io::stdout().flush();
    // todo: clear stdin?
    
    let mut child = cmd.spawn()?;
    while child.try_wait()?.is_none() {}

    Ok(())
}