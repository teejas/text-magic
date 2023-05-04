use editor::Editor;
use crossterm::{execute, cursor, terminal};

mod editor;

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Failed to disable raw-mode for terminal");
        execute!(
            std::io::stdout(), 
            terminal::Clear(terminal::ClearType::All)
        ).expect("Error clearing the screen on exit");
        execute!(
            std::io::stdout(), 
            cursor::MoveTo(0,0),
        ).expect("Error resetting the cursor to (0,0) on exit");
    }
}

fn main() -> crossterm::Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;
    let mut editor = Editor::new();
    while editor.run()? {};
    Ok(())
}
