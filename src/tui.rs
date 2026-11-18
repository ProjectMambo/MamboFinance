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
        widgets::{Paragraph, Row, Tabs, Widget},
    },
    selection_table,
};

use crate::{Account, AccountKind, Application, Capabilities, CreateAccount};

const TAB_TITLES: [&str; 5] = ["Overview", "Transactions", "Budgets", "Insights", "Imports"];

const ACCOUNT_KINDS: [AccountKind; 5] = [
    AccountKind::Bank,
    AccountKind::Card,
    AccountKind::Cash,
    AccountKind::Wallet,
    AccountKind::Marketplace,
];

#[derive(Debug, Default, Eq, PartialEq)]
enum View {
    #[default]
    Main,
    Accounts,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum AccountField {
    #[default]
    Name,
    Kind,
    Currency,
}

#[derive(Debug, Eq, PartialEq)]
enum FormAction {
    Continue,
    Cancel,
    Submit,
}

#[derive(Debug, Default)]
struct AccountForm {
    name: String,
    kind_index: usize,
    currency: String,
    field: AccountField,
}

impl AccountForm {
    fn handle_key(&mut self, code: KeyCode) -> FormAction {
        match code {
            KeyCode::Esc => return FormAction::Cancel,
            KeyCode::Enter => return FormAction::Submit,
            KeyCode::Tab => self.field = self.field.next(),
            KeyCode::BackTab => self.field = self.field.previous(),
            KeyCode::Left | KeyCode::Up if self.field == AccountField::Kind => {
                self.kind_index = self.kind_index.saturating_sub(1);
            }
            KeyCode::Right | KeyCode::Down if self.field == AccountField::Kind => {
                self.kind_index = (self.kind_index + 1).min(ACCOUNT_KINDS.len() - 1);
            }
            KeyCode::Backspace => match self.field {
                AccountField::Name => {
                    self.name.pop();
                }
                AccountField::Currency => {
                    self.currency.pop();
                }
                AccountField::Kind => {}
            },
            KeyCode::Char(character) => match self.field {
                AccountField::Name if self.name.chars().count() < 80 => {
                    self.name.push(character);
                }
                AccountField::Currency if self.currency.len() < 3 => {
                    self.currency.push(character.to_ascii_uppercase());
                }
                AccountField::Name | AccountField::Kind | AccountField::Currency => {}
            },
            _ => {}
        }
        FormAction::Continue
    }

    fn input(&self) -> CreateAccount {
        CreateAccount {
            name: self.name.clone(),
            kind: ACCOUNT_KINDS[self.kind_index],
            currency: self.currency.clone(),
        }
    }
}

impl AccountField {
    const fn next(self) -> Self {
        match self {
            Self::Name => Self::Kind,
            Self::Kind => Self::Currency,
            Self::Currency => Self::Name,
        }
    }

    const fn previous(self) -> Self {
        match self {
            Self::Name => Self::Currency,
            Self::Kind => Self::Name,
            Self::Currency => Self::Kind,
        }
    }
}

#[derive(Debug, Default)]
struct UiState {
    selected_tab: usize,
    view: View,
    accounts: Vec<Account>,
    account_form: Option<AccountForm>,
    error: Option<String>,
    notice: Option<String>,
}

/// Runs the keyboard-only `MamboUI` foundation shell.
///
/// # Errors
///
/// Returns an error when storage status, terminal setup, rendering, or input fails.
pub fn run(application: &Application) -> Result<(), Box<dyn Error>> {
    let capabilities = application.capabilities()?;
    let mut terminal = TerminalSession::start()?;
    let mut state = UiState {
        accounts: application.list_accounts()?.accounts,
        ..UiState::default()
    };

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
        if handle_key(&mut state, key.code, application)? {
            break;
        }
    }
    Ok(())
}

fn handle_key(
    state: &mut UiState,
    code: KeyCode,
    application: &Application,
) -> Result<bool, Box<dyn Error>> {
    if let Some(form) = &mut state.account_form {
        state.error = None;
        match form.handle_key(code) {
            FormAction::Continue => {}
            FormAction::Cancel => state.account_form = None,
            FormAction::Submit => match application.create_account(&form.input()) {
                Ok(account) => {
                    state.account_form = None;
                    state.accounts = application.list_accounts()?.accounts;
                    state.notice = Some(format!("Created {} ({})", account.name, account.currency));
                }
                Err(error) => state.error = Some(error.to_string()),
            },
        }
        return Ok(false);
    }

    match state.view {
        View::Main => match code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('m') => {
                state.accounts = application.list_accounts()?.accounts;
                state.view = View::Accounts;
                state.notice = None;
            }
            KeyCode::Char(character @ '1'..='5') => {
                state.selected_tab = character.to_digit(10).unwrap_or(1) as usize - 1;
            }
            KeyCode::Left => state.selected_tab = state.selected_tab.saturating_sub(1),
            KeyCode::Right => {
                state.selected_tab = (state.selected_tab + 1).min(TAB_TITLES.len() - 1);
            }
            _ => {}
        },
        View::Accounts => match code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Esc | KeyCode::Char('m') => {
                state.view = View::Main;
                state.notice = None;
            }
            KeyCode::Char('a') => {
                state.account_form = Some(AccountForm::default());
                state.error = None;
                state.notice = None;
            }
            KeyCode::Char('r') => {
                state.accounts = application.list_accounts()?.accounts;
                state.notice = Some("Accounts refreshed".to_owned());
            }
            _ => {}
        },
    }
    Ok(false)
}

fn render(frame: &mut Frame<'_>, state: &UiState, capabilities: &Capabilities) {
    let theme = Theme::default();
    let content = Shell::new("MamboFinance")
        .subtitle("Experimental account foundation")
        .footer("Accounts are available; ledger entries arrive in a later slice")
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

    match state.view {
        View::Main => render_main(frame, state, tabs_area, body_area, &theme),
        View::Accounts => render_accounts(frame, state, tabs_area, body_area, &theme),
    }
    render_status(frame, state, capabilities, status_area, &theme);
    render_help(frame, state, help_area, &theme);
}

fn render_main(
    frame: &mut Frame<'_>,
    state: &UiState,
    tabs_area: mambo_ui::ratatui::layout::Rect,
    body_area: mambo_ui::ratatui::layout::Rect,
    theme: &Theme,
) {
    Tabs::new(TAB_TITLES)
        .select(state.selected_tab)
        .style(theme.muted())
        .highlight_style(theme.selected())
        .divider(" │ ")
        .render(tabs_area, frame.buffer_mut());

    let panel = Panel::new(TAB_TITLES[state.selected_tab])
        .focused(true)
        .block(theme);
    let inner = panel.inner(body_area);
    frame.render_widget(panel, body_area);
    let (title, message) = if state.selected_tab == 0 {
        (
            "Ledger foundation ready",
            "Open Manage to create and inspect accounts.",
        )
    } else {
        (
            "Not implemented yet",
            "This domain arrives in a later development-plan stage.",
        )
    };
    frame.render_widget(EmptyState::new(title, theme).message(message), inner);
}

fn render_accounts(
    frame: &mut Frame<'_>,
    state: &UiState,
    title_area: mambo_ui::ratatui::layout::Rect,
    body_area: mambo_ui::ratatui::layout::Rect,
    theme: &Theme,
) {
    frame.render_widget(
        Paragraph::new("Manage › Accounts").style(theme.title()),
        title_area,
    );
    let panel = Panel::new(if state.account_form.is_some() {
        "New account"
    } else {
        "Accounts"
    })
    .focused(true)
    .block(theme);
    let inner = panel.inner(body_area);
    frame.render_widget(panel, body_area);

    if let Some(form) = &state.account_form {
        let marker = |field| if form.field == field { ">" } else { " " };
        frame.render_widget(
            Paragraph::new(format!(
                "{} Name: {}\n{} Kind: {}\n{} Currency: {}",
                marker(AccountField::Name),
                form.name,
                marker(AccountField::Kind),
                ACCOUNT_KINDS[form.kind_index],
                marker(AccountField::Currency),
                form.currency
            )),
            inner,
        );
    } else if state.accounts.is_empty() {
        frame.render_widget(
            EmptyState::new("No accounts yet", theme)
                .message("Press a to create the first account."),
            inner,
        );
    } else {
        let rows = state.accounts.iter().map(|account| {
            Row::new([
                account.name.clone(),
                account.kind.to_string(),
                account.currency.clone(),
            ])
        });
        frame.render_widget(
            selection_table(
                rows,
                [
                    Constraint::Min(16),
                    Constraint::Length(12),
                    Constraint::Length(8),
                ],
                theme,
            )
            .header(Row::new(["Name", "Kind", "Currency"]).style(theme.title())),
            inner,
        );
    }
}

fn render_status(
    frame: &mut Frame<'_>,
    state: &UiState,
    capabilities: &Capabilities,
    area: mambo_ui::ratatui::layout::Rect,
    theme: &Theme,
) {
    let (status, message) = if let Some(error) = &state.error {
        (Status::Error, error.clone())
    } else if let Some(notice) = &state.notice {
        (Status::Success, notice.clone())
    } else {
        (
            Status::Info,
            format!(
                "SQLite schema v{}  {} active account(s)",
                capabilities.database_schema_version,
                state.accounts.len()
            ),
        )
    };
    frame.render_widget(StatusLine::new(status, message, theme), area);
}

fn render_help(
    frame: &mut Frame<'_>,
    state: &UiState,
    area: mambo_ui::ratatui::layout::Rect,
    theme: &Theme,
) {
    let hints = if state.account_form.is_some() {
        vec![
            KeyHint::new("Tab", "field"),
            KeyHint::new("←/→", "kind"),
            KeyHint::new("Enter", "create"),
            KeyHint::new("Esc", "cancel"),
        ]
    } else if state.view == View::Accounts {
        vec![
            KeyHint::new("a", "add"),
            KeyHint::new("r", "refresh"),
            KeyHint::new("Esc", "back"),
            KeyHint::new("q", "quit"),
        ]
    } else {
        vec![
            KeyHint::new("1-5", "tabs"),
            KeyHint::new("←/→", "navigate"),
            KeyHint::new("m", "manage"),
            KeyHint::new("q", "quit"),
        ]
    };
    frame.render_widget(Help::new(hints, theme), area);
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
            database_schema_version: 2,
            interfaces: vec!["mamboui_tui".to_owned(), "mcp_streamable_http".to_owned()],
            available_tools: vec![
                "get_capabilities".to_owned(),
                "list_accounts".to_owned(),
                "create_account".to_owned(),
            ],
            unavailable_domains: vec!["account_update_archive_restore".to_owned()],
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
        assert!(text.contains("SQLite schema v2"));
    }

    #[test]
    fn reduced_terminal_does_not_panic() {
        let backend = TestBackend::new(20, 4);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render(frame, &UiState::default(), &capabilities()))
            .expect("draw");
    }

    #[test]
    fn account_form_collects_all_required_fields() {
        let mut form = AccountForm::default();
        for character in "Daily".chars() {
            assert_eq!(
                form.handle_key(KeyCode::Char(character)),
                FormAction::Continue
            );
        }
        form.handle_key(KeyCode::Tab);
        form.handle_key(KeyCode::Right);
        form.handle_key(KeyCode::Tab);
        for character in "usd".chars() {
            form.handle_key(KeyCode::Char(character));
        }

        assert_eq!(form.handle_key(KeyCode::Enter), FormAction::Submit);
        assert_eq!(
            form.input(),
            CreateAccount {
                name: "Daily".to_owned(),
                kind: AccountKind::Card,
                currency: "USD".to_owned(),
            }
        );
    }

    #[test]
    fn account_form_creates_through_the_shared_application() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let application =
            Application::open(directory.path().join("finance.db")).expect("application");
        let mut state = UiState {
            view: View::Accounts,
            account_form: Some(AccountForm {
                name: "Wallet".to_owned(),
                kind_index: 3,
                currency: "SGD".to_owned(),
                field: AccountField::Currency,
            }),
            ..UiState::default()
        };

        assert!(!handle_key(&mut state, KeyCode::Enter, &application).expect("key"));

        assert!(state.account_form.is_none());
        assert_eq!(state.accounts.len(), 1);
        assert_eq!(state.accounts[0].kind, AccountKind::Wallet);
        assert_eq!(
            application.list_accounts().expect("accounts").accounts,
            state.accounts
        );
    }

    #[test]
    fn rejected_account_input_remains_editable() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let application =
            Application::open(directory.path().join("finance.db")).expect("application");
        let mut state = UiState {
            view: View::Accounts,
            account_form: Some(AccountForm {
                name: "Wallet".to_owned(),
                kind_index: 3,
                currency: "XX".to_owned(),
                field: AccountField::Currency,
            }),
            ..UiState::default()
        };

        handle_key(&mut state, KeyCode::Enter, &application).expect("key");

        let form = state.account_form.as_ref().expect("preserved form");
        assert_eq!(form.name, "Wallet");
        assert_eq!(form.currency, "XX");
        assert!(
            state.error.as_deref().is_some_and(|error| {
                error.contains("currency") && error.contains("three-letter")
            })
        );
        assert!(state.accounts.is_empty());

        let backend = TestBackend::new(20, 4);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render(frame, &state, &capabilities()))
            .expect("reduced form draw");
    }
}
