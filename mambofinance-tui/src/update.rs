use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use mambofinance_lib::user::{Category, Currency, Fund, Group, Transaction}; // Import all fetchables

use crate::app::App;
use crate::widgets::Navigable;
use crate::widgets::query_table::QueryTable;
use crate::widgets::user_list::MENU_ITEMS;
use crate::widgets::{Focusable, side_bar::SideBar, user_list::UserList};

pub fn handle_events(app: &mut App) -> Result<()> {
    if let Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        ..
    }) = event::read()?
    {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            KeyCode::Left | KeyCode::Right => handle_focus_movement(app, code),
            KeyCode::Up | KeyCode::Down => handle_pane_navigation(app, code),
            _ => {}
        }
    }
    Ok(())
}

/// Handles switching panel focus back and forth (Left and Right arrows)
fn handle_focus_movement(app: &mut App, key_code: KeyCode) {
    // Instantiate a temporary UserList context manager to execute clamped state step shifts
    let mut user_list = UserList::new(&app.user, &mut app.user_list_state);

    // 2. Call methods without arguments
    match key_code {
        KeyCode::Left => user_list.focus_previous(),
        KeyCode::Right => user_list.focus_next(),
        _ => {}
    }
}

/// Handles vertical scrolling inside whichever panel is currently active (Up and Down arrows)
fn handle_pane_navigation(app: &mut App, key_code: KeyCode) {
    if app.user_list_state.current_pane == 0 {
        // --- SIDEBAR FOCUS ACTIVE ---
        let sidebar = SideBar::new(Vec::from(MENU_ITEMS));
        match key_code {
            KeyCode::Down => sidebar.next(&mut app.user_list_state.sidebar_state),
            KeyCode::Up => sidebar.previous(&mut app.user_list_state.sidebar_state),
            _ => {}
        }
    } else {
        // --- DATA TABLE FOCUS ACTIVE ---
        // Look at what menu tab index is currently highlighted to build the correct data runner
        let active_tab = app.user_list_state.sidebar_state.selected().unwrap_or(0);

        match active_tab {
            0 => {
                if let Ok(table) = QueryTable::<Transaction>::new(&app.user) {
                    if key_code == KeyCode::Down {
                        table.next(&mut app.user_list_state.table_state);
                    } else {
                        table.previous(&mut app.user_list_state.table_state);
                    }
                }
            }
            1 => {
                if let Ok(table) = QueryTable::<Group>::new(&app.user) {
                    if key_code == KeyCode::Down {
                        table.next(&mut app.user_list_state.table_state);
                    } else {
                        table.previous(&mut app.user_list_state.table_state);
                    }
                }
            }
            2 => {
                if let Ok(table) = QueryTable::<Category>::new(&app.user) {
                    if key_code == KeyCode::Down {
                        table.next(&mut app.user_list_state.table_state);
                    } else {
                        table.previous(&mut app.user_list_state.table_state);
                    }
                }
            }
            3 => {
                if let Ok(table) = QueryTable::<Fund>::new(&app.user) {
                    if key_code == KeyCode::Down {
                        table.next(&mut app.user_list_state.table_state);
                    } else {
                        table.previous(&mut app.user_list_state.table_state);
                    }
                }
            }
            4 => {
                if let Ok(table) = QueryTable::<Currency>::new(&app.user) {
                    if key_code == KeyCode::Down {
                        table.next(&mut app.user_list_state.table_state);
                    } else {
                        table.previous(&mut app.user_list_state.table_state);
                    }
                }
            }
            _ => {}
        }
    }
}
