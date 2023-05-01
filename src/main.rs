use editor::Editor;
use crossterm::{execute, terminal};

mod editor;

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Failed to disable raw-mode for terminal");
        execute!(
            std::io::stdout(), 
            terminal::Clear(terminal::ClearType::All)
        ).expect("Error clearing the screen on exit");
    }
}

fn main() -> crossterm::Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;
    let mut editor = Editor::new();
    while editor.run()? {}
    Ok(())
}
