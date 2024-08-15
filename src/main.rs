use core::str;
use std::env;
use std::env::home_dir;
use std::fmt::Display;
use std::fs::read_to_string;
use std::io::{Read, Write};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use actions::{parse_actions, Action};
use anathema::backend::tui::Screen;
use anathema::component::{ComponentId, Emitter};
use anathema::prelude::*;
use command::{run_command, run_command_quiet};
use crossterm::cursor::MoveTo;
use crossterm::style::{Color, Colored, ContentStyle, Stylize};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
};
use crossterm::{cursor, ExecutableCommand};
use fake_editor::{Doc, Editor};
use quittable_backend::{QuittableTuiBackend, SHOULD_QUIT};
use rand::Rng;
use syntect::highlighting::ThemeSet;

use self::instruction::Instruction;

mod actions;
mod command;
mod fake_editor;
mod file_tree;
mod instruction;
mod parse;
mod quittable_backend;
pub(crate) mod syntax;

/// -----------------------
/// CONFIGURATION
/// -----------------------

const TYPING_DELAY_RANGE_MS: Range<u64> = 35..85;

/// -----------------------

fn sleep_between_characters() {
    let sleep = rand::thread_rng().gen_range(TYPING_DELAY_RANGE_MS);
    thread::sleep(Duration::from_millis(sleep));
}

fn insts(lines: Box<[syntax::Line<'_>]>) -> Vec<Instruction> {
    let mut instructions = parse::Parser::new(lines).instructions();
    instructions.pop();
    instructions.insert(0, Instruction::Pause(1000));
    instructions.push(Instruction::WaitForQuit);
    instructions
}

fn enable_tui() {
    let mut output = std::io::stdout();

    let _ = output.execute(EnterAlternateScreen);
    let _ = enable_raw_mode();
    _ = output.execute(cursor::Hide);
}

fn disable_tui() {
    let mut screen = Screen::new((0, 0));
    let _ = screen.restore(std::io::stdout());
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap();

    let action_file = read_to_string(&path).unwrap();
    let actions = parse_actions(action_file);

    let mut output = std::io::stdout();
    _ = output.execute(MoveTo(0, 0));
    _ = output.execute(Clear(ClearType::All));

    let theme = ThemeSet::get_theme("themes/custom.stTheme").unwrap();
    let base_path = env::current_dir().expect("Failed to get current directory");

    wait_for_input();
    for action in actions.iter() {
        match action {
            Action::ChangeDir(dir) => {
                print_fake_cmd();
                write_command(["cd", &**dir].iter().map(|el| *el));
                cd(&**dir);
                wait_for_input();
            }
            Action::ChangeDirQuiet(dir) => {
                cd(&**dir);
            }
            Action::RunCommand(cmd) => {
                print_fake_cmd();
                write_command(cmd.iter().map(|el| &**el));
                match run_command(cmd) {
                    Ok(_) => (),
                    Err(e) => eprintln!("{}", ContentStyle::default().red().apply(e)),
                }
                wait_for_input();
            }
            Action::RunCommandOnlyOutput(cmd) => match run_command(cmd) {
                Ok(_) => (),
                Err(e) => eprintln!("{}", ContentStyle::default().red().apply(e)),
            },
            Action::RunCommandQuiet(cmd) => match run_command_quiet(cmd) {
                Ok(_) => (),
                Err(e) => eprintln!("{}", ContentStyle::default().red().apply(e)),
            },
            Action::RunEditor(dst, src) => {
                print_fake_cmd();
                write_command(["edit", dst].iter().map(|el| *el));
                let Ok(dir) = env::current_dir() else {
                    panic!("Could not acquire current directory")
                };
                let dst = dir.join(&**dst);
                let src = dir.join(&**src);
                match std::fs::copy(&src, &dst) {
                    Ok(_) => (),
                    Err(e) => panic!(
                        "Failed to copy from {} to {}: {e}",
                        src.display(),
                        dst.display()
                    ),
                }

                let Some(extension) = dst.extension() else {
                    panic!("File {} does not have an extension", dst.display())
                };
                let extension = match str::from_utf8(extension.as_encoded_bytes()) {
                    Ok(v) => v,
                    Err(e) => panic!(
                        "{}: Failed to convert utf-8 to string: {e:?}!",
                        dst.display()
                    ),
                };

                let code = read_to_string(&dst).unwrap();
                let spans = syntax::highlight(&code, extension, &theme);
                let instructions = insts(spans);

                let mut runtime = Runtime::builder(
                    Document::new("@main"),
                    QuittableTuiBackend(TuiBackend::builder().finish().unwrap()),
                );

                let editor_state = Doc::new(dst);

                let current_dir = env::current_dir().expect("Failed to get working directory");
                env::set_current_dir(&base_path).expect("Failed to set working directory to app root");
                
                let (tx, rx) = mpsc::channel();
                let cid = runtime
                .register_component("main", "components/index.aml", Editor::new(tx), editor_state)
                .unwrap();
            runtime
                    .register_component("footer", "components/footer.aml", (), ())
                    .unwrap();
                runtime
                    .register_component("folder_list", "components/folder_list.aml", (), ())
                    .unwrap();
                
                run_editor(cid, runtime.finish().expect("Failed to build runtime"), rx, instructions);
                
                env::set_current_dir(current_dir).expect("Failed to restore working directory");
            }
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    sleep_between_characters();
    sleep_between_characters();
}

fn wait_for_input() {
    let mut input = std::io::stdin();
    enable_raw_mode().unwrap();

    loop {
        let mut buf = [0u8; 4];
        let Ok(num_read) = input.read(&mut buf) else {
            continue;
        };
        if num_read > 0 {
            for n in &buf[0..num_read] {
                if *n != 0 {
                    disable_raw_mode().unwrap();
                    return;
                }
            }
        }
    }
}

fn cd(path: &str) {
    if path.starts_with('/') {
        env::set_current_dir(PathBuf::from(path)).expect("Failed to set current dir");
    } else if path.starts_with('~') {
        let Some(homedir) = home_dir() else {
            panic!("Could not acquire home directory")
        };
        env::set_current_dir(homedir.join(path)).expect("Failed to set current dir");
    } else {
        let Ok(dir) = env::current_dir() else {
            panic!("Could not acquire current directory")
        };
        env::set_current_dir(dir.join(path)).expect("Failed to set current dir");
    }
}

struct FakeCmdPrinter;

impl Display for FakeCmdPrinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = env::current_dir().unwrap_or_else(|_| PathBuf::new());
        match home_dir() {
            None => path.display().fmt(f),
            Some(v) => match path.strip_prefix(v) {
                Err(..) => path.display().fmt(f),
                Ok(v) => {
                    f.write_str("~/")?;
                    v.display().fmt(f)
                }
            },
        }?;
        f.write_str(" $ ")
    }
}

fn print_fake_cmd() {
    print!("{}", ContentStyle::default().green().apply(FakeCmdPrinter));
    _ = std::io::stdout().flush();
}

fn write_command<'a>(mut cmd: impl Iterator<Item = &'a str>) {
    let Some(cmd_name) = cmd.next() else {
        return;
    };

    print!("\x1b[{}m", Colored::ForegroundColor(Color::Cyan));
    write_str_typing(cmd_name);

    print!("\x1b[{}m", Colored::ForegroundColor(Color::Reset));
    let mut output = std::io::stdout();
    _ = output.flush();
    for arg in cmd {
        print!(" ");
        _ = output.flush();
        sleep_between_characters();
        write_str_typing(arg);
    }
    print!("\n");
    _ = std::io::stdout().flush();
}

fn write_str_typing(value: &str) {
    let mut output = std::io::stdout();
    for c in value.chars() {
        let mut data = [0u8; 4];
        _ = output.write(c.encode_utf8(&mut data).as_bytes());
        _ = output.flush();
        sleep_between_characters();
    }
}

fn run_editor(
    cid: ComponentId<Instruction>,
    mut runtime: Runtime<QuittableTuiBackend>,
    rx: Receiver<()>,
    instructions: Vec<Instruction>,
) {
    let emitter = runtime.emitter();

    thread::spawn(move || {
        for i in instructions {
            if let Instruction::Pause(ms) = i {
                thread::sleep(Duration::from_millis(ms));
                continue;
            }

            if let Instruction::Wait = i {
                _ = emitter.emit(cid, i);
                _ = rx.recv();
                continue;
            }

            if let Instruction::WaitForQuit = i {
                _ = rx.recv();
                if let Ok(mut should_quit) = SHOULD_QUIT.lock() {
                    *should_quit = true;
                }
                return;
            }
            _ = rx.try_recv();

            sleep_between_characters();
            _ = emitter.emit(cid, i);
        }

        _ = rx.recv();
        if let Ok(mut should_quit) = SHOULD_QUIT.lock() {
            *should_quit = true;
        }
    });

    enable_tui();
    runtime.run();
    disable_tui();
}
