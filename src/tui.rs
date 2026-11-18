use std::{error::Error, io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use mambo_ui::{
    EmptyState, Help, KeyHint, Panel, Shell, Status, StatusLine, Theme,
    ratatui::{
        Frame, Terminal,
        backend::{Backend, CrosstermBackend},
        layout::{Constraint, Layout},
        widgets::{Tabs, Widget},
    },
};

use crate::{Application, Capabilities};

const TAB_TITLES: [&str; 5] = ["Overview", "Transactions", "Budgets", "Insights", "Imports"];

#[derive(Debug, Default)]
struct UiState {
    selected_tab: usize,
}

/// Runs the keyboard-only `MamboUI` foundation shell.
///
/// # Errors
///
/// Returns an error when storage status, terminal setup, rendering, or input fails.
pub fn run(application: &Application) -> Result<(), Box<dyn Error>> {
    let capabilities = application.capabilities()?;
    let mut terminal = TerminalSession::start()?;
    let mut state = UiState::default();

    loop {
        terminal
            .terminal
            .draw(|frame| render(frame, &state, &capabilities))?;
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') => break,
            KeyCode::Char(character @ '1'..='5') => {
                state.selected_tab = character.to_digit(10).unwrap_or(1) as usize - 1;
            }
            KeyCode::Left => {
                state.selected_tab = state.selected_tab.saturating_sub(1);
            }
            KeyCode::Right => {
                state.selected_tab = (state.selected_tab + 1).min(TAB_TITLES.len() - 1);
            }
            _ => {}
        }
    }
    Ok(())
}

fn render(frame: &mut Frame<'_>, state: &UiState, capabilities: &Capabilities) {
    let theme = Theme::default();
    let content = Shell::new("MamboFinance")
        .subtitle("Experimental ledger foundation")
        .footer("No ledger actions are enabled in this first slice")
        .render(frame, &theme);
    if content.is_empty() {
        return;
    }

    let [tabs_area, body_area, status_area, help_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(content);

    Tabs::new(TAB_TITLES)
        .select(state.selected_tab)
        .style(theme.muted())
        .highlight_style(theme.selected())
        .divider(" │ ")
        .render(tabs_area, frame.buffer_mut());

    let panel = Panel::new(TAB_TITLES[state.selected_tab])
        .focused(true)
        .block(&theme);
    let inner = panel.inner(body_area);
    frame.render_widget(panel, body_area);
    let (title, message) = if state.selected_tab == 0 {
        (
            "Ledger foundation ready",
            "Durable storage and the read-only MCP capability tool are available.",
        )
    } else {
        (
            "Not implemented yet",
            "This domain arrives in a later development-plan stage.",
        )
    };
    frame.render_widget(EmptyState::new(title, &theme).message(message), inner);

    frame.render_widget(
        StatusLine::new(
            Status::Info,
            format!(
                "SQLite schema v{}  MCP tool: get_capabilities",
                capabilities.database_schema_version
            ),
            &theme,
        ),
        status_area,
    );
    frame.render_widget(
        Help::new(
            [
                KeyHint::new("1-5", "tabs"),
                KeyHint::new("←/→", "navigate"),
                KeyHint::new("q", "quit"),
            ],
            &theme,
        ),
        help_area,
    );
}

struct TerminalSession<B: Backend> {
    terminal: Terminal<B>,
}

impl TerminalSession<CrosstermBackend<io::Stdout>> {
    fn start() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(error);
        }
        match Terminal::new(CrosstermBackend::new(stdout)) {
            Ok(terminal) => Ok(Self { terminal }),
            Err(error) => {
                let _ = disable_raw_mode();
                let _ = execute!(io::stdout(), LeaveAlternateScreen);
                Err(error)
            }
        }
    }
}

impl<B: Backend> Drop for TerminalSession<B> {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mambo_ui::ratatui::{Terminal, backend::TestBackend};

    fn capabilities() -> Capabilities {
        Capabilities {
            contract_version: "0.1".to_owned(),
            database_schema_version: 1,
            interfaces: vec!["mamboui_tui".to_owned(), "mcp_streamable_http".to_owned()],
            available_tools: vec!["get_capabilities".to_owned()],
            unavailable_domains: vec!["accounts".to_owned()],
        }
    }

    #[test]
    fn foundation_screen_renders_at_target_size() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render(frame, &UiState::default(), &capabilities()))
            .expect("draw");

        let text = terminal.backend().to_string();
        assert!(text.contains("MamboFinance"));
        assert!(text.contains("Ledger foundation ready"));
        assert!(text.contains("SQLite schema v1"));
    }

    #[test]
    fn reduced_terminal_does_not_panic() {
        let backend = TestBackend::new(20, 4);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render(frame, &UiState::default(), &capabilities()))
            .expect("draw");
    }
}
