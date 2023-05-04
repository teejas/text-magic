use position::CursorController;
use writing::{StatusMessage, WritingController};
use file::{FileController, Row};
use std::{cmp, io};
use std::io::Write;
use crossterm::event::*;
use crossterm::{cursor, queue, style, terminal};
use std::path::PathBuf;

mod position;
mod file;
mod writing;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_QUIT_ATTEMPTS: u64 = 3;

#[derive(Default)]
pub struct Controllers {
    writing_ctrlr: WritingController,
    cursor_ctrlr: CursorController,
    file_ctrlr: FileController,
    status_msg: StatusMessage,
    dirty: u64,
    quit_attempts: u64,
}

impl Drop for Controllers {
    fn drop(&mut self) {
        if self.is_dirty() && self.quit_attempts < MAX_QUIT_ATTEMPTS { // if dirty and not intentionally quit, save a .tmp file
            match &self.file_ctrlr.filename {
                Some(name) => {
                    let mut new_filename = name.clone();
                    new_filename.set_extension("tmp");
                    self.file_ctrlr
                        .save_file(&new_filename)
                        .expect("Failed to save emergency .tmp file on crash");
                },
                None => {
                    self.file_ctrlr
                        .save_file(&PathBuf::from("./tm-crashed.tmp"))
                        .expect("Failed to save emergency .tmp file on crash");
                }
            }
        }
    }
}

impl Controllers {
    pub fn new() -> Self {
        let win_size = terminal::size()
            .map(|(x,y)| (x as usize, y as usize - 2))
            .unwrap();
        Self {
            writing_ctrlr: WritingController::new(),
            cursor_ctrlr: CursorController::new(win_size),
            file_ctrlr: FileController::new(),
            status_msg: StatusMessage::default(),
            dirty: 0,
            quit_attempts: 0,
        }
    }
    
    fn draw_rows(&mut self) -> io::Result<()> {
        for i in 0..self.cursor_ctrlr.editor_height {
            let file_row = self.cursor_ctrlr.row_offset + i;
            if file_row >= self.file_ctrlr.count_rows() {
                if self.file_ctrlr.filename.is_none() && i == self.cursor_ctrlr.editor_height / 4 {
                    let welcome = format!("Text Magic editor -- Version {}", VERSION);
                    let len = cmp::min(welcome.len(), self.cursor_ctrlr.editor_width);
                    let mut padding = (self.cursor_ctrlr.editor_width - welcome.len()) / 2;
                    if padding != 0 {
                        self.writing_ctrlr.push('~');
                        padding -= 1;
                        (0..padding).for_each(|_| self.writing_ctrlr.push(' '));
                    }
                    self.writing_ctrlr.push_str(&welcome[..len])
                } else {
                    self.writing_ctrlr.push('~');
                }
            } else {
                let row = self.file_ctrlr.get_render(file_row);
                let column_offset = self.cursor_ctrlr.column_offset;
                let len = cmp::min(row.len().saturating_sub(column_offset), self.cursor_ctrlr.editor_width);
                let start = if len == 0 { 0 } else { column_offset };
                self.writing_ctrlr.push_str(&row[start..start+len])
            }
            queue!(
                self.writing_ctrlr,
                terminal::Clear(terminal::ClearType::UntilNewLine)
            )?;
            self.writing_ctrlr.push_str("\r\n")
        }
        Ok(())
    }

    fn draw_status_bar(&mut self) {
        self.writing_ctrlr.push_str(&style::Attribute::Reverse.to_string());
        let info = format!(
            "{} {} -- {} lines",
            self.file_ctrlr
                .filename
                .as_ref()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("[No Name]"),
            if self.dirty > 0 { "(modified)" } else { "" },
            self.file_ctrlr.count_rows()
        );
        let info_len = cmp::min(info.len(), self.cursor_ctrlr.editor_width);
        self.writing_ctrlr.push_str(&info[..info_len]);
        let line_info = format!(
            "{}/{}",
            self.cursor_ctrlr.cursor_y + 1,
            self.file_ctrlr.count_rows()
        );
        for i in info_len..self.cursor_ctrlr.editor_width {
            if self.cursor_ctrlr.editor_width - i == line_info.len() {
                self.writing_ctrlr.push_str(&line_info);
                break
            } else {
                self.writing_ctrlr.push(' ');
            }
        }
        self.writing_ctrlr.push_str(&style::Attribute::Reset.to_string());
        self.writing_ctrlr.push_str("\r\n");
    }

    fn draw_message_bar(&mut self) {
        queue!(
            self.writing_ctrlr,
            terminal::Clear(terminal::ClearType::UntilNewLine)
        ).unwrap();
        if let Some(msg) = self.status_msg.message() {
            let len = cmp::min(self.cursor_ctrlr.editor_width, msg.len());
            self.writing_ctrlr.push_str(&msg[..len])
        }
    }
    
    pub fn refresh_screen(&mut self) -> crossterm::Result<()> {
        self.cursor_ctrlr.scroll(&self.file_ctrlr);
        let (mut x, mut y) = self.cursor_ctrlr.pos();
        x -= self.cursor_ctrlr.column_offset;
        y -= self.cursor_ctrlr.row_offset;
        queue!(
            self.writing_ctrlr,
            cursor::Hide,
            cursor::MoveTo(0, 0)
        )?;
        self.draw_rows()?;
        self.draw_status_bar();
        self.draw_message_bar();
        queue!(
            self.writing_ctrlr, 
            cursor::MoveTo(x as u16, y as u16),
            cursor::Show
        )?;
        self.writing_ctrlr.flush()
    }

    pub fn insert_char(&mut self, ch: char) {
        if self.cursor_ctrlr.cursor_y == self.file_ctrlr.count_rows() {
            self.file_ctrlr.insert_row(
                self.cursor_ctrlr.cursor_y,
                String::new()
            );
            self.file_ctrlr
                .get_editor_row_mut(self.cursor_ctrlr.cursor_y)
                .insert_char(self.cursor_ctrlr.cursor_x, ch);
            self.cursor_ctrlr.cursor_x += 1;
        } else if self.cursor_ctrlr.cursor_x >= self.cursor_ctrlr.editor_width - 1 {
            let curr_row = self.file_ctrlr
                .get_editor_row_mut(self.cursor_ctrlr.cursor_y);
            curr_row.insert_char(self.cursor_ctrlr.cursor_x, ch);
            let mut trunc_len = 0;
            let new_row_content = if ch == ' ' || ch == '\t' {
                String::new()
            } else {
                match curr_row.content.split_whitespace().last() {
                    None => String::new(),
                    Some(this) => {
                        if this.len() < curr_row.len() {
                            trunc_len = curr_row.len() - this.len();
                            String::from(this)
                        } else { // no whitespace in row
                            String::new()
                        }
                    }
                }
            };
            if trunc_len > 0 {
                curr_row.content.truncate(trunc_len);
                FileController::render_row(curr_row);
            }
            self.cursor_ctrlr.cursor_x = new_row_content.len();
            self.cursor_ctrlr.cursor_y += 1;
            self.file_ctrlr.insert_row(
                self.cursor_ctrlr.cursor_y,
                new_row_content 
            );
        } else {
            self.file_ctrlr
                .get_editor_row_mut(self.cursor_ctrlr.cursor_y)
                .insert_char(self.cursor_ctrlr.cursor_x, ch);
            self.cursor_ctrlr.cursor_x += 1;
        }
        self.dirty += 1;
    }

    pub fn insert_newline(&mut self) {
        if self.cursor_ctrlr.cursor_x == 0 {
            self.file_ctrlr.insert_row(self.cursor_ctrlr.cursor_y, String::new());
        } else {
            let curr_row = self.file_ctrlr
                .get_editor_row_mut(self.cursor_ctrlr.cursor_y);
            let new_row_content: String = curr_row.content[self.cursor_ctrlr.cursor_x..].into();
            curr_row.content.truncate(self.cursor_ctrlr.cursor_x);
            FileController::render_row(curr_row);
            self.file_ctrlr.insert_row(self.cursor_ctrlr.cursor_y + 1, new_row_content)
        }
        self.cursor_ctrlr.cursor_x = 0;
        self.cursor_ctrlr.cursor_y += 1;
        self.dirty += 1
    }

    pub fn delete_char(&mut self, shift: KeyModifiers) {
        if self.cursor_ctrlr.cursor_y == self.file_ctrlr.count_rows()
            || (self.cursor_ctrlr.cursor_x == 0 && self.cursor_ctrlr.cursor_y == 0) {
            return
        }
        match shift {
            KeyModifiers::SHIFT => {
                self.delete_prev_word()
            }
            _ => {
                let row = self
                    .file_ctrlr
                    .get_editor_row_mut(self.cursor_ctrlr.cursor_y);
                if self.cursor_ctrlr.cursor_x > 0 {
                    self.cursor_ctrlr.cursor_x -= 1;
                    row.delete_char(self.cursor_ctrlr.cursor_x);
                    self.dirty += 1;
                } else {
                    let len_prev_row = self.file_ctrlr.get_editor_row(self.cursor_ctrlr.cursor_y - 1).len();
                    self.file_ctrlr.join_adjacent_rows(self.cursor_ctrlr.cursor_y);
                    self.cursor_ctrlr.cursor_x = len_prev_row;
                    self.cursor_ctrlr.cursor_y -= 1;
                }
            }
        }
    }

    fn delete_prev_word(&mut self) { // deletes word in same row but behind the cursor_x, uses same logic as SHIFT+LEFT
        let curr_row = self.file_ctrlr.get_editor_row(self.cursor_ctrlr.cursor_y);
        let move_len = if self.cursor_ctrlr.cursor_x == 0 {
            1
        } else {
            curr_row.content[0..self.cursor_ctrlr.cursor_x]
                .split_whitespace()
                .last()
                .unwrap()
                .len() + 1 // add one to put cursor in the next whitepsace
        };
        for _ in 0..move_len {
            self.delete_char(KeyModifiers::NONE)
        };
    }

    pub fn move_cursor(&mut self, key: KeyCode, shift: KeyModifiers) {
        match key {
            KeyCode::PageUp => {
                self.cursor_ctrlr.cursor_y = self.cursor_ctrlr.row_offset;
                for _ in 0..self.cursor_ctrlr.editor_height {
                    self.cursor_ctrlr.move_cursor(
                        KeyCode::Up, &self.file_ctrlr
                    );
                }
            }
            KeyCode::PageDown => {
                self.cursor_ctrlr.cursor_y = cmp::min(
                    self.cursor_ctrlr.editor_height + self.cursor_ctrlr.row_offset - 1,
                    self.file_ctrlr.count_rows()
                );
                for _ in 0..self.cursor_ctrlr.editor_height {
                    self.cursor_ctrlr.move_cursor(
                        KeyCode::Down, &self.file_ctrlr
                    );
                }
            }
            _ => {
                if self.cursor_ctrlr.cursor_y >= self.file_ctrlr.count_rows() 
                    || shift != KeyModifiers::SHIFT {
                    self.cursor_ctrlr.move_cursor(
                        key, &self.file_ctrlr
                    );
                    return
                }
                if shift == KeyModifiers::SHIFT {
                    let curr_row: &Row = self.file_ctrlr.get_editor_row(self.cursor_ctrlr.cursor_y);
                    let move_len = match key {
                        KeyCode::Up | KeyCode::Down => 5,
                        KeyCode::Right => {
                            if self.cursor_ctrlr.cursor_y >= self.file_ctrlr.count_rows() 
                                || self.cursor_ctrlr.cursor_x >= curr_row.content.len() {
                                1
                            } else {
                                match curr_row.content[self.cursor_ctrlr.cursor_x..].split_whitespace().next() {
                                    None => 1,
                                    Some(word) => {
                                        word.len() + 1 // add one to put cursor in the next whitespace
                                    }
                                }
                            }
                        }
                        KeyCode::Left => {
                            if self.cursor_ctrlr.cursor_y >= self.file_ctrlr.count_rows() 
                                || self.cursor_ctrlr.cursor_x == 0 {
                                1
                            } else {
                                match curr_row.content[0..self.cursor_ctrlr.cursor_x].split_whitespace().last() {
                                    None => 1,
                                    Some(word) => {
                                        word.len() + 1 // add one to put cursor in the next whitespace
                                    }
                                }
                            }
                        }
                        _ => 1
                    };
                    for _ in 0..move_len {
                        self.cursor_ctrlr.move_cursor(
                            key, &self.file_ctrlr
                        );
                    }
                }
            }
        }
    }

    pub fn save(&mut self) -> io::Result<usize> {
        self.dirty = 0;
        self.file_ctrlr.save()
    }

    pub fn set_status_msg(&mut self, s: String) {
        self.status_msg.set_message(s);
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty > 0
    }

    pub fn attempt_to_quit(&mut self) -> bool {
        if self.is_dirty() && self.quit_attempts < MAX_QUIT_ATTEMPTS {
            self.set_status_msg(
                format!(
                    "WARNING! File has unsaved changes. \
                    Press Ctrl+C {} more times to quit without saving or Ctrl+S to save first.",
                    MAX_QUIT_ATTEMPTS - self.quit_attempts
                )
            );
            self.quit_attempts += 1;
            false // not quitting
        } else {
            true // quitting
        }
    }

    pub fn loaded_from_file(&self) -> bool {
        self.file_ctrlr.filename.is_some()
    }

    pub fn set_filename(&mut self, filename: Option<PathBuf>) {
        self.file_ctrlr.filename = filename
    }
}