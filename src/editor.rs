use controllers::Controllers;
use std::time;
use crossterm::event::*;
use crossterm::event;

mod controllers;

#[macro_export]
macro_rules! prompt {
    ($ctrlrs:expr, $($args:tt)*) => {{
        let ctrlrs: &mut Controllers = &mut $ctrlrs;
        let mut input = String::with_capacity(32);
        loop {
            ctrlrs.set_status_msg(format!($($args)*, input));
            ctrlrs.refresh_screen()?;
            match InputReader.read_key()? {
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    if !input.is_empty() {
                        ctrlrs.set_status_msg(String::new());
                        break;
                    }
                },
                KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    ctrlrs.set_status_msg(String::new());
                    input.clear();
                    break;
                },
                KeyEvent {
                    code: KeyCode::Backspace | KeyCode::Delete,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    input.pop();
                },
                KeyEvent {
                    code: key @ (KeyCode::Char(..) | KeyCode::Tab),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    ..
                } => {
                    input.push(match key {
                        KeyCode::Tab => '\t',
                        KeyCode::Char(ch) => ch,
                        _ => unreachable!()
                    })
                },
                _ => {}
            }
        }
        if input.is_empty() { None } else { Some(input) }
    }};
}

struct InputReader;

impl InputReader {
    fn read_key(&self) -> crossterm::Result<KeyEvent> {
        loop {
            if event::poll(time::Duration::from_millis(5000))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
    }
}

pub struct Editor {
    reader: InputReader,
    ctrlrs: Controllers
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        Self { 
            reader: InputReader,
            ctrlrs: Controllers::new()
        }
    }

    fn process_keypress(&mut self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => {
                if !self.ctrlrs.attempt_to_quit(){
                    return Ok(true)
                };
                return Ok(false)
            },
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => {
                if !self.ctrlrs.loaded_from_file() {
                    let filepath = prompt!(&mut self.ctrlrs, "Save as: {}").map(|it| it.into());
                    if filepath == None {
                        self.ctrlrs.set_status_msg("Save aborted!".into());
                        return Ok(true)
                    }
                    self.ctrlrs.set_filename(filepath);
                }
                self.ctrlrs.save().map(|len| {
                    self.ctrlrs.set_status_msg(format!("{} bytes written to disk", len));
                })?;
            },
            KeyEvent {
                code: key @ (KeyCode::Backspace | KeyCode::Delete),
                modifiers: event::KeyModifiers::NONE,
                ..
            } => {
                if key == KeyCode::Delete {
                    self.ctrlrs.move_cursor(KeyCode::Right);
                }
                self.ctrlrs.delete_char();
            },
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: event::KeyModifiers::NONE,
                ..
            } => self.ctrlrs.insert_newline(),
            KeyEvent {
                code: direction @ (
                    KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::End
                    | KeyCode::Home
                    | KeyCode::PageUp
                    | KeyCode::PageDown
                ),
                modifiers: event::KeyModifiers::NONE,
                ..
            } => self.ctrlrs.move_cursor(direction),
            KeyEvent {
                code: code @ (KeyCode::Char(..) | KeyCode::Tab),
                modifiers: event::KeyModifiers::NONE | event::KeyModifiers::SHIFT,
                ..
            } => {
                self.ctrlrs.insert_char(
                    match code {
                        KeyCode::Tab => '\t',
                        KeyCode::Char(ch) => ch,
                        _ => unreachable!()
                    }
                )
            }
            _ => {}
        }
        Ok(true)
    }

    pub fn run(&mut self) -> crossterm::Result<bool> {
        self.ctrlrs.refresh_screen()?;
        self.process_keypress()
    }
}