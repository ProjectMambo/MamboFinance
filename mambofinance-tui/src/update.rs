use crate::app::App;
use crate::widgets::FocusState;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

pub fn handle_events(app: &mut App) -> Result<()> {
    if let Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        ..
    }) = event::read()?
    {
        match code {
            KeyCode::Char('q') => {
                app.should_quit = true;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                app.ui_state.focus_next();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.ui_state.focus_prev();
            }
            KeyCode::Up | KeyCode::Char('k') => app.ui_state.prev(),
            KeyCode::Down | KeyCode::Char('j') => app.ui_state.next(),
            _ => {}
        }
    }
    Ok(())
}
