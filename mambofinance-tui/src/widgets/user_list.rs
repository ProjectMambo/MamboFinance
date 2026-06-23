use mambofinance_lib::user::{Category, Currency, Fund, Group, Transaction, User};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{ListState, StatefulWidget, TableState, Widget},
};

use crate::widgets::{Focusable, query_table::QueryTable, side_bar::SideBar};

pub const MENU_ITEMS: &[&str] = &[
    "Transactions",
    "Groups",
    "Categories",
    "Funds",
    "Currencies",
];

pub struct UserList<'a> {
    pub user: &'a User,
    pub state: &'a mut UserListState,
}

impl<'a> UserList<'a> {
    pub fn new(user: &'a User, state: &'a mut UserListState) -> Self {
        Self { user, state }
    }
}

impl<'a> Focusable for UserList<'a> {
    fn focus_next(&mut self) {
        self.state.current_pane = Self::next_clamp(2, self.state.current_pane);
    }

    fn focus_previous(&mut self) {
        self.state.current_pane = Self::prev_clamp(self.state.current_pane);
    }
}

impl<'a> Widget for UserList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let menu_items = Vec::from(MENU_ITEMS);
        let max_sidebar_width = menu_items.iter().map(|s| s.len()).max().unwrap_or(12) + 6;

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(max_sidebar_width as u16),
                Constraint::Min(0),
            ])
            .split(area);

        let sidebar = SideBar::new(menu_items);

        StatefulWidget::render(sidebar, chunks[0], buf, &mut self.state.sidebar_state);

        let active_tab = self.state.sidebar_state.selected().unwrap_or(0);
        let table_area = chunks[1];

        match active_tab {
            0 => {
                if let Ok(table) = QueryTable::<Transaction>::new(self.user) {
                    StatefulWidget::render(table, table_area, buf, &mut self.state.table_state);
                }
            }
            1 => {
                if let Ok(table) = QueryTable::<Group>::new(self.user) {
                    StatefulWidget::render(table, table_area, buf, &mut self.state.table_state);
                }
            }
            2 => {
                if let Ok(table) = QueryTable::<Category>::new(self.user) {
                    StatefulWidget::render(table, table_area, buf, &mut self.state.table_state);
                }
            }
            3 => {
                if let Ok(table) = QueryTable::<Fund>::new(self.user) {
                    StatefulWidget::render(table, table_area, buf, &mut self.state.table_state);
                }
            }
            4 => {
                if let Ok(table) = QueryTable::<Currency>::new(self.user) {
                    StatefulWidget::render(table, table_area, buf, &mut self.state.table_state);
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct UserListState {
    pub current_pane: usize,      // 0 = Sidebar, 1 = Table View
    pub sidebar_state: ListState, // Tracks active sub-tab index
    pub table_state: TableState,  // Tracks active table row index
}

impl UserListState {
    pub fn new() -> Self {
        let mut sidebar_state = ListState::default();
        sidebar_state.select(Some(0));

        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Self {
            current_pane: 0,
            sidebar_state,
            table_state,
        }
    }
}
