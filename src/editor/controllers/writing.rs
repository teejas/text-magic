use std::time;
use std::io::{self, stdout};

pub struct StatusMessage {
    message: Option<String>,
    set_time: Option<time::Instant>
}

impl Default for StatusMessage {
    fn default() -> Self {
        Self::new("Help: Ctrl+S = Save / Ctrl+C = Quit".into())
    }
}

impl StatusMessage {
    pub fn new(initial_message: String) -> Self {
        Self {
            message: Some(initial_message),
            set_time: Some(time::Instant::now()),
        }
    }

    pub fn set_message(&mut self, message: String) {
        self.message = Some(message);
        self.set_time = Some(time::Instant::now())
    }

    pub fn message(&mut self) -> Option<&String> {
        self.set_time.and_then(|time| {
            if time.elapsed() > time::Duration::from_secs(5) {
                self.message = None;
                self.set_time = None;
                None
            } else {
                Some(self.message.as_ref().unwrap())
            }
        })
    }
}

pub struct WritingController {
    content: String
}

impl Default for WritingController {
    fn default() -> Self {
        Self::new()
    }
}

impl WritingController {
    pub fn new() -> Self {
        Self {
            content: String::new()
        }
    }

    pub fn push(&mut self, ch: char) {
        self.content.push(ch)
    }

    pub fn push_str(&mut self, string: &str) {
        self.content.push_str(string)
    }
}

impl io::Write for WritingController {
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