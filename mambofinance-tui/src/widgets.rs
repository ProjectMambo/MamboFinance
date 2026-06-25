use std::fmt::Debug;

use mambofinance_lib::user::{User, UserError};
use ratatui::{Frame, layout::Rect};

use crate::widgets::user_list::UserListState;

pub mod query_table;
pub mod side_bar;
pub mod user_list;

pub trait PanelState: Debug {
    fn next_wrapped(&self, len: usize, cur: usize) -> usize {
        if cur >= len - 1 { 0 } else { cur + 1 }
    }

    fn prev_wrapped(&self, len: usize, cur: usize) -> usize {
        if cur == 0 { len - 1 } else { cur - 1 }
    }

    fn next(&mut self);
    fn prev(&mut self);
    fn none(&mut self);
    fn selected(&self) -> Option<usize>;
}

pub trait FocusState: Debug {
    fn focus_next(&mut self);
    fn focus_prev(&mut self);
    fn next(&mut self);
    fn prev(&mut self);
}

#[derive(Debug)]
pub enum TabState {
    UserList(UserListState),
}

impl TabState {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        match self {
            TabState::UserList(state) => {
                frame.render_stateful_widget(state.to_widget(), area, state);
            }
        }
    }

    pub fn update_data(&mut self, user: &User) -> Result<(), UserError> {
        match self {
            TabState::UserList(state) => state.update_data(user),
        }
    }
}

impl FocusState for TabState {
    fn focus_next(&mut self) {
        match self {
            TabState::UserList(state) => state.focus_next(),
        }
    }

    fn focus_prev(&mut self) {
        match self {
            TabState::UserList(state) => state.focus_prev(),
        }
    }

    fn next(&mut self) {
        match self {
            TabState::UserList(state) => state.next(),
        }
    }

    fn prev(&mut self) {
        match self {
            TabState::UserList(state) => state.prev(),
        }
    }
}

#[derive(Debug)]
pub struct UIState {
    current_tab: usize,
    pub tabs: Vec<TabState>,
    total_tabs: usize,
}

impl UIState {
    pub fn new(tabs: Vec<TabState>) -> Self {
        let total_tabs = tabs.len();
        UIState {
            current_tab: 0,
            tabs,
            total_tabs,
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut TabState> {
        self.tabs.get_mut(self.current_tab)
    }

    pub fn tab_next(&mut self) {
        self.current_tab = if self.total_tabs == 0 {
            0
        } else {
            (self.current_tab + 1).min(self.total_tabs - 1)
        };
    }

    pub fn tab_prev(&mut self) {
        self.current_tab = self.current_tab.saturating_sub(1);
    }
}

impl FocusState for UIState {
    fn focus_next(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.current_tab) {
            tab.focus_next();
        }
    }

    fn focus_prev(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.current_tab) {
            tab.focus_prev();
        }
    }

    fn next(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.current_tab) {
            tab.next()
        }
    }

    fn prev(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.current_tab) {
            tab.prev()
        }
    }
}
