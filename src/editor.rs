use crate::editor::controller::Controller;
use std::time::Duration;
use crossterm::event::*;
use crossterm::event;

pub mod controller;
struct Reader;

impl Reader {
    fn read_key(&self) -> crossterm::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(5000))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
    }
}

pub struct Editor {
    reader: Reader,
    ctrlr: Controller
}

impl Editor {
    pub fn new() -> Self {
        Self { 
            reader: Reader,
            ctrlr: Controller::new()
        }
    }

    fn process_keypress(&mut self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => return Ok(false),
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
            } => self.ctrlr.move_cursor(direction),
            KeyEvent {
                code: key_code,
                ..
            } => println!("{:?}\r", key_code)
        }
        Ok(true)
    }

    pub fn run(&mut self) -> crossterm::Result<bool> {
        self.ctrlr.refresh_screen()?;
        self.process_keypress()
    }
}