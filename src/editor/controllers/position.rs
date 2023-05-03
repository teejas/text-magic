use crate::editor::controllers::file::{Row, FileController};
use std::cmp;
use crossterm::event::*;

const TAB_STOP: usize = 4;

pub struct CursorController {
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub editor_height: usize,
    pub editor_width: usize,
    pub row_offset: usize,
    pub column_offset: usize,
    render_x: usize
}

impl Default for CursorController {
    fn default() -> Self {
        Self::new((0,0))
    }
}

impl CursorController {
    pub fn new(win_size: (usize, usize)) -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            editor_height: win_size.1,
            editor_width: if win_size.0 > 100 { // if editor is too wide, divide in half
                win_size.0 / 2 + 1
            } else { 
                win_size.0 
            },
            row_offset: 0,
            column_offset: 0,
            render_x: 0
        }
    }

    pub fn pos(&self) -> (usize, usize) {
        (self.render_x, self.cursor_y)
    }

    pub fn get_render_x(&self, row: &Row) -> usize {
        row.content[..self.cursor_x]
            .chars()
            .fold(0, |render_x, c| {
                if c == '\t' {
                    render_x + (TAB_STOP - 1) - (render_x % TAB_STOP) + 1
                } else {
                    render_x + 1
                }
            })
    }

    pub fn scroll(&mut self, editor_rows: &FileController) {
        self.render_x = 0;
        if self.cursor_y < editor_rows.count_rows() {
            self.render_x = self.get_render_x(editor_rows.get_editor_row(self.cursor_y))
        }
        self.row_offset = cmp::min(self.row_offset, self.cursor_y);
        if self.cursor_y >= self.row_offset + self.editor_height {
            self.row_offset = self.cursor_y - self.editor_height + 1
        }
        self.column_offset = cmp::min(self.column_offset, self.cursor_x);
        if self.cursor_x >= self.column_offset + self.editor_width {
            self.column_offset = self.cursor_x - self.editor_width + 1
        }
    }

    pub fn move_cursor(&mut self, direction: KeyCode, editor_rows: &FileController) {
        match direction {
            KeyCode::Up => if self.cursor_y != 0 { self.cursor_y -= 1 },
            KeyCode::Left => {
                if self.cursor_x != 0 { 
                    self.cursor_x -= 1 
                } else if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                    self.cursor_x = editor_rows.get_editor_row(self.cursor_y).len();
                }
            }
            KeyCode::Down => if self.cursor_y < editor_rows.count_rows() { self.cursor_y += 1 },
            KeyCode::Right => {
                if self.cursor_y < editor_rows.count_rows() {
                    if self.cursor_x < editor_rows.get_editor_row(self.cursor_y).len() {
                        self.cursor_x += 1
                    } else {
                        self.cursor_y += 1;
                        self.cursor_x = 0
                    }
                }
            }
            KeyCode::End => {
                if self.cursor_y < editor_rows.count_rows() {
                    self.cursor_x = editor_rows.get_editor_row(self.cursor_y).len();
                }
            }
            KeyCode::Home => {
                self.cursor_x = 0;
            }
            _ => unimplemented!(),
        }
        let row_len = if self.cursor_y < editor_rows.count_rows() {
            editor_rows.get_editor_row(self.cursor_y).len()
        } else {
            0
        };
        self.cursor_x = cmp::min(self.cursor_x, row_len);
    }
}