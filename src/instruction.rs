use anathema::state::Hex;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Type(char, bool),
    SetForeground(Hex),
    Newline { x: i32 },
    SetX(i32),
    Pause(u64),
    Wait,
    WaitForQuit,
}
