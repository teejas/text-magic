use std::{cmp, env, fs};
use std::path::Path;
use std::io::{self, stdout, Write};
use crossterm::event::*;
use crossterm::{cursor, queue, terminal};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Controller {
    editor_contents: EditorContents,
    cursor_ctrlr: CursorController,
    editor_rows: EditorRows
}

impl Controller {
    pub fn new() -> Self {
        let win_size = terminal::size()
            .map(|(x,y)| (x as usize, y as usize))
            .unwrap();
        Self {
            editor_contents: EditorContents::new(),
            cursor_ctrlr: CursorController::new(win_size),
            editor_rows: EditorRows::new()
        }
    }
    
    fn draw_rows(&mut self) -> io::Result<()> {
        for i in 0..self.cursor_ctrlr.editor_height {
            let file_row = self.cursor_ctrlr.row_offset + i;
            if file_row >= self.editor_rows.count_rows() {
                if !self.editor_rows.loaded_file && i == self.cursor_ctrlr.editor_height / 4 {
                    let welcome = format!("Text Magic editor -- Version {}", VERSION);
                    let len = cmp::min(welcome.len(), self.cursor_ctrlr.editor_width);
                    let mut padding = (self.cursor_ctrlr.editor_width - welcome.len()) / 2;
                    if padding != 0 {
                        self.editor_contents.push('~');
                        padding -= 1;
                        (0..padding).for_each(|_| self.editor_contents.push(' '));
                    }
                    self.editor_contents.push_str(&welcome[..len])
                } else {
                    self.editor_contents.push('~');
                }
            } else {
                let row = self.editor_rows.get_row(file_row);
                let column_offset = self.cursor_ctrlr.column_offset;
                let len = cmp::min(row.len().saturating_sub(column_offset), self.cursor_ctrlr.editor_width);
                let start = if len == 0 { 0 } else { column_offset };
                self.editor_contents.push_str(&row[start..start+len])
            }
            queue!(
                self.editor_contents,
                terminal::Clear(terminal::ClearType::UntilNewLine)
            )?;
            if i < self.cursor_ctrlr.editor_height - 1 {
                self.editor_contents.push_str("\r\n")
            }
        }
        Ok(())
    }
    
    pub fn refresh_screen(&mut self) -> crossterm::Result<()> {
        self.cursor_ctrlr.scroll();
        let (mut x, mut y) = self.cursor_ctrlr.pos();
        x -= self.cursor_ctrlr.column_offset;
        y -= self.cursor_ctrlr.row_offset;
        queue!(
            self.editor_contents,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        self.draw_rows()?;
        queue!(
            self.editor_contents, 
            cursor::MoveTo(x as u16, y as u16),
            cursor::Show
        )?;
        self.editor_contents.flush()
    }

    pub fn move_cursor(&mut self, key: KeyCode) {
        for _ in 0..self.cursor_ctrlr.editor_height {
            match key {
                KeyCode::PageUp => self.cursor_ctrlr.move_cursor(KeyCode::Up, &self.editor_rows),
                KeyCode::PageDown => self.cursor_ctrlr.move_cursor(KeyCode::Down, &self.editor_rows),
                _ => {
                    self.cursor_ctrlr.move_cursor(key, &self.editor_rows);
                    break
                }
            }
        }
    }
}

struct EditorRows {
    row_contents: Vec<Box<str>>,
    loaded_file: bool
}

impl EditorRows {
    fn new() -> Self {
        let mut arg = env::args();

        match arg.nth(1) {
            None => Self {
                row_contents: Vec::new(),
                loaded_file: false
            },
            Some(file) => Self::from_file(file.as_ref())
        }
    }

    fn from_file(file: &Path) -> Self {
        let file_contents = fs::read_to_string(file).expect("Failed to read file");
        Self {
            row_contents: file_contents.lines().map(|it| it.into()).collect(),
            loaded_file: true
        }
    }

    fn count_rows(&self) -> usize {
        self.row_contents.len()
    }

    fn get_row(&self, idx: usize) -> &str {
        &self.row_contents[idx]
    }
}

struct CursorController {
    cursor_x: usize,
    cursor_y: usize,
    editor_height: usize,
    editor_width: usize,
    row_offset: usize,
    column_offset: usize
}

impl CursorController {
    fn new(win_size: (usize, usize)) -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            editor_height: win_size.1,
            editor_width: win_size.0,
            row_offset: 0,
            column_offset: 0
        }
    }

    fn pos(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }

    fn scroll(&mut self) {
        self.row_offset = cmp::min(self.row_offset, self.cursor_y);
        if self.cursor_y >= self.row_offset + self.editor_height {
            self.row_offset = self.cursor_y - self.editor_height + 1
        }
        self.column_offset = cmp::min(self.column_offset, self.cursor_x);
        if self.cursor_x >= self.column_offset + self.editor_width {
            self.column_offset = self.cursor_x - self.editor_width + 1
        }
    }

    fn move_cursor(&mut self, direction: KeyCode, editor_rows: &EditorRows) {
        match direction {
            KeyCode::Up => {
                if self.cursor_y != 0 { self.cursor_y -= 1 }
            }
            KeyCode::Left => {
                if self.cursor_x != 0 { 
                    self.cursor_x -= 1 
                } else if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                    self.cursor_x = editor_rows.get_row(self.cursor_y).len();
                }
            }
            KeyCode::Down => {
                if self.cursor_y < editor_rows.count_rows() { self.cursor_y += 1 }
            }
            KeyCode::Right => {
                if self.cursor_y < editor_rows.count_rows() {
                    if self.cursor_x < editor_rows.get_row(self.cursor_y).len() {
                        self.cursor_x += 1
                    } else {
                        self.cursor_y += 1;
                        self.cursor_x = 0
                    }
                }
            }
            KeyCode::End => {
                self.cursor_x = self.editor_width - 1;
            }
            KeyCode::Home => {
                self.cursor_x = 0;
            }
            _ => unimplemented!(),
        }
        let row_len = if self.cursor_y < editor_rows.count_rows() {
            editor_rows.get_row(self.cursor_y).len()
        } else {
            0
        };
        self.cursor_x = cmp::min(self.cursor_x, row_len);
    }
}

struct EditorContents {
    content: String
}

impl EditorContents {
    fn new() -> Self {
        Self {
            content: String::new()
        }
    }

    fn push(&mut self, ch: char) {
        self.content.push(ch)
    }

    fn push_str(&mut self, string: &str) {
        self.content.push_str(string)
    }
}

impl io::Write for EditorContents {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.content.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(io::ErrorKind::WriteZero.into())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let out = write!(stdout(), "{}", self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}