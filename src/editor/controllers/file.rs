use std::{env, fs};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const TAB_STOP: usize = 4;

pub struct Row {
    pub content: String,
    pub render: String
}

impl Default for Row {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}

impl Row {
    fn new(content: String, render: String) -> Self {
        Self {
            content,
            render
        }
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn insert_char(&mut self, idx: usize, ch: char) {
        self.content.insert(idx, ch);
        FileController::render_row(self);
    }

    pub fn delete_char(&mut self, idx: usize) {
        self.content.remove(idx);
        FileController::render_row(self);
    }
}

pub struct FileController {
    rows: Vec<Row>,
    pub filename: Option<PathBuf>
}

impl Default for FileController {
    fn default() -> Self {
        Self::new()
    }
}

impl FileController {
    pub fn new() -> Self {
        let mut arg = env::args();

        match arg.nth(1) {
            None => Self {
                rows: Vec::new(),
                filename: None
            },
            Some(file) => Self::from_file(file.into())
        }
    }

    pub fn render_row(row: &mut Row) {
        let mut index = 0;
        let capacity = row
            .content
            .chars()
            .fold(0, |acc, next| acc + (if next == '\t' { 8 } else { 1 }));
        row.render = String::with_capacity(capacity);
        row.content.chars().for_each(|c| {
            index += 1;
            if c == '\t' {
                row.render.push(' ');
                while index % TAB_STOP != 0 {
                    row.render.push(' ');
                    index += 1
                }
            } else {
                row.render.push(c)
            }
        })
    }

    fn from_file(file: PathBuf) -> Self {
        let fp: &Path = file.as_path();
        let file_contents = if fp.exists() {
            fs::read_to_string(&file).expect("Failed to read file")                
                .lines()
                .map(|it| {
                    let mut row = Row::new(it.into(), String::new());
                    Self::render_row(&mut row);
                    row
                })
                .collect()
        } else {
            Vec::new()
        };
        Self {
            rows: file_contents,
            filename: Some(file)
        }
    }

    pub fn get_render(&self, idx: usize) -> &String {
        &self.rows[idx].render
    }

    pub fn count_rows(&self) -> usize {
        self.rows.len()
    }

    pub fn get_editor_row(&self, idx: usize) -> &Row {
        &self.rows[idx]
    }

    pub fn get_editor_row_mut(&mut self, idx: usize) -> &mut Row {
        &mut self.rows[idx]
    }

    pub fn join_adjacent_rows(&mut self, row_idx: usize) {
        if row_idx == 0 { return }
        let curr_row = self.rows.remove(row_idx);
        let prev_row = self.get_editor_row_mut(row_idx - 1);
        prev_row.content.push_str(&curr_row.content);
        Self::render_row(prev_row);
    }

    pub fn insert_row(&mut self, row_idx: usize, content: String) {
        let mut new_row = Row::new(content, String::new());
        Self::render_row(&mut new_row);
        self.rows.insert(row_idx, new_row);
    }

    fn save_file(&self, filename: &PathBuf) -> io::Result<usize> {
        let mut file = fs::OpenOptions::new().write(true).create(true).open(filename)?;
        let contents: String = self.rows
            .iter()
            .map(|it| it.content.as_str())
            .collect::<Vec<&str>>()
            .join("\n");
        file.set_len(contents.len() as u64)?;
        file.write_all(contents.as_bytes())?;
        Ok(contents.as_bytes().len())
    }

    pub fn save(&self) -> io::Result<usize> {
        match &self.filename {
            None => Err(io::Error::new(io::ErrorKind::Other, "No filename specified")),
            Some(name) => {
                self.save_file(name)
            }
        }
    }
}