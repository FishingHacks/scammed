use std::{path::PathBuf, sync::mpsc::Sender};

use anathema::state::Hex;

#[derive(Debug, Clone)]
pub enum Instruction {
    MoveCursor(u16, u16),
    Type(char, bool),
    SetForeground(Hex),
    Newline { x: i32 },
    SetX(i32),
    Pause(u64),
    Wait,
    WaitForQuit,
    UpdateState(PathBuf, Sender<()>),
    HideCursor,
}
