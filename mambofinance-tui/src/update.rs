use crate::{
    app::{App, AppContext},
    widgets::PanelState,
};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

pub fn handle_events(app: &mut App) -> Result<()> {
    if let Event::Key(key_event) = event::read()? {
        if key_event.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true;
            }
            _ => app.ui_state.handle_key_events(
                key_event,
                AppContext {
                    user: &mut app.user,
                    input_override: &app.input_override,
                },
            ),
        }
    }
    Ok(())
}
