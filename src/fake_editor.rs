use std::{env, path::PathBuf, sync::mpsc::Sender};

use anathema::component::*;
use anathema::default_widgets::{CanvasAttribs, Overflow};
use anathema::geometry::{Pos, Size};
use anathema::prelude::Context;
use anathema::state::Hex;

use crate::file_tree::empty_folder;
use crate::{
    file_tree::{get_path_list, read_file_tree, Folder},
    instruction::Instruction,
};

#[derive(State)]
struct Line {
    spans: Value<List<Span>>,
}

impl Line {
    pub fn empty() -> Self {
        Self {
            spans: List::empty(),
        }
    }
}

#[derive(State)]
struct Span {
    text: Value<char>,
    bold: Value<bool>,
    foreground: Value<Hex>,
}

impl Span {
    pub fn new(c: char, foreground: Hex, bold: bool) -> Self {
        Self {
            text: c.into(),
            foreground: foreground.into(),
            bold: bold.into(),
        }
    }

    pub fn empty() -> Self {
        Self {
            text: ' '.into(),
            foreground: Hex::from((255, 255, 255)).into(),
            bold: false.into(),
        }
    }
}

#[derive(State)]
pub struct Doc {
    doc_height: Value<usize>,
    screen_cursor_x: Value<i32>,
    screen_cursor_y: Value<i32>,
    buf_cursor_x: Value<i32>,
    buf_cursor_y: Value<i32>,
    lines: Value<List<Line>>,
    current_instruction: Value<Option<String>>,
    title: Value<String>,
    waiting: Value<String>,
    show_cursor: Value<bool>,
    tree: Value<Folder>,
    folder_list: Value<List<String>>,
    file_name: Value<String>,
}

impl Doc {
    pub fn new(mut focused: PathBuf) -> Self {
        let dir = env::current_dir().unwrap();
        if !focused.is_absolute() {
            focused = dir.join(focused);
        }

        let title = focused
            .strip_prefix(&dir)
            .unwrap_or(&focused)
            .to_str()
            .expect("Failed to turn path into valid UTF-8")
            .to_string()
            .into();

        let tree = read_file_tree(&dir, &focused).into();

        let mut folder_list = get_path_list(&dir, focused);
        let file_name = match folder_list.pop_back() {
            Some(v) => v,
            None => String::new().into(),
        };

        Self {
            doc_height: 1.into(),
            screen_cursor_x: 0.into(),
            screen_cursor_y: 0.into(),
            buf_cursor_x: 0.into(),
            buf_cursor_y: 0.into(),
            lines: List::from_iter(vec![Line::empty()]),
            current_instruction: None.into(),
            title,
            waiting: false.to_string().into(),
            show_cursor: true.into(),
            tree,
            folder_list,
            file_name,
        }
    }

    pub fn update_state(&mut self, new_title: String, mut new_focused: PathBuf) {
        while self.lines.len() > 0 {
            self.lines.pop_back();
        }

        let dir = env::current_dir().unwrap();
        if !new_focused.is_absolute() {
            new_focused = dir.join(new_focused);
        }
        let tree = read_file_tree(&dir, &new_focused);

        let mut folder_list = get_path_list(&dir, new_focused);
        match folder_list.pop_back() {
            Some(v) => self.file_name = v,
            Some(..) => (),
            None => self.file_name.to_mut().clear(),
        }

        *self.title.to_mut() = new_title;
        *self.tree.to_mut() = tree;
        self.folder_list = folder_list;
    }
}

pub struct Editor {
    cursor: Pos,
    cell_attribs: CanvasAttribs,
    foreground: Hex,
    instructions: Vec<Instruction>,
    ack: Sender<()>,
}

impl Editor {
    pub fn new(ack: Sender<()>) -> Self {
        Self {
            cursor: Pos::ZERO,
            cell_attribs: CanvasAttribs::new(),
            foreground: Hex::from((255, 255, 255)),
            instructions: vec![],
            ack,
        }
    }

    fn update_cursor(&mut self, state: &mut Doc, overflow: &mut Overflow, size: Size) {
        // Make sure there are enough lines and spans
        while self.cursor.y as usize >= state.lines.len() {
            state.lines.push_back(Line::empty());
        }

        {
            let mut lines = state.lines.to_mut();
            let line = lines.get_mut(self.cursor.y as usize).unwrap();

            let spans = &mut line.to_mut().spans;
            while self.cursor.x as usize > spans.len() {
                spans.push_back(Span::empty());
            }
        }

        let mut screen_cursor = self.cursor - overflow.offset();

        if screen_cursor.y < 0 {
            overflow.scroll_up_by(-screen_cursor.y);
            screen_cursor.y = 0;
        }

        if screen_cursor.y >= size.height as i32 {
            let offset = screen_cursor.y + 1 - size.height as i32;
            overflow.scroll_down_by(offset);
            screen_cursor.y = size.height as i32 - 1;
        }

        state.screen_cursor_x.set(screen_cursor.x);
        state.screen_cursor_y.set(screen_cursor.y);
        state.buf_cursor_x.set(self.cursor.x);
        state.buf_cursor_y.set(self.cursor.y);
    }

    fn apply_inst(&mut self, inst: Instruction, doc: &mut Doc, mut elements: Elements<'_, '_>) {
        doc.current_instruction.set(Some(format!("{inst:?}")));
        elements.query().by_tag("overflow").first(|el, _| {
            let size = el.size();
            let vp = el.to::<Overflow>();

            match &inst {
                Instruction::MoveCursor(x, y) => {
                    self.cursor.x = *x as i32;
                    self.cursor.y = *y as i32;
                    self.update_cursor(doc, vp, size);
                }
                Instruction::Type(c, bold) => {
                    {
                        let mut lines = doc.lines.to_mut();
                        let line = lines.get_mut(self.cursor.y as usize).unwrap();
                        let mut line = line.to_mut();
                        line.spans.insert(
                            self.cursor.x as usize,
                            Span::new(*c, self.foreground, *bold),
                        );
                        self.cursor.x += 1;
                    }

                    self.update_cursor(doc, vp, size);
                }
                Instruction::SetForeground(hex) => self.foreground = *hex,
                Instruction::Newline { x } => {
                    self.cursor.x = *x;
                    self.cursor.y += 1;
                    self.update_cursor(doc, vp, size);
                }
                Instruction::SetX(x) => {
                    self.cursor.x = *x as i32;
                    self.update_cursor(doc, vp, size);
                }
                Instruction::Pause(_) => unreachable!(),
                Instruction::Wait => doc.waiting.set(true.to_string()),
                Instruction::HideCursor => {
                    doc.show_cursor.set(false);
                }
                Instruction::WaitForQuit => {}
                Instruction::UpdateState(new_focused, new_transmitter) => {
                    self.ack = new_transmitter.clone();
                    doc.update_state(
                        new_focused.display().to_string().into(),
                        new_focused.clone(),
                    );
                }
            }
        });
    }
}

impl Component for Editor {
    type Message = Instruction;
    type State = Doc;

    fn on_key(
        &mut self,
        key: KeyEvent,
        state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        _: Context<'_>,
    ) {
        state.waiting.set(false.to_string());
        self.ack.send(());
    }

    fn message(
        &mut self,
        inst: Self::Message,
        state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        _: Context<'_>,
    ) {
        self.apply_inst(inst, state, elements);
    }
}
