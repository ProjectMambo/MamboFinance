use std::fmt::Debug;

use crossterm::event::{KeyCode, KeyEvent};
use mambofinance_lib::user::{User, UserError};
use ratatui::{Frame, layout::Rect};

use crate::{
    app::AppContext,
    widgets::user_list::{UserList, UserListState},
};

pub mod bottom_bar;
pub mod pop_up;
pub mod query_table;
pub mod side_bar;
pub mod user_list;

pub trait Actionable: Debug {
    fn next_wrapped(len: usize) -> impl Fn(usize) -> usize {
        move |cur| {
            if cur >= len.saturating_sub(1) {
                0
            } else {
                cur + 1
            }
        }
    }

    fn prev_wrapped(len: usize) -> impl Fn(usize) -> usize {
        move |cur| {
            if cur == 0 {
                len.saturating_sub(1)
            } else {
                cur - 1
            }
        }
    }

    fn next_capped(len: usize) -> impl Fn(usize) -> usize {
        move |cur| (cur + 1).min(len.saturating_sub(1))
    }

    fn prev_capped() -> impl Fn(usize) -> usize {
        |cur| cur.saturating_sub(1)
    }

    fn next(&mut self) {
        if self.is_empty() {
            return;
        }
        let i = self.selected().map_or(0, Self::next_wrapped(self.len()));
        self.select(Some(i));
    }

    fn prev(&mut self) {
        if self.is_empty() {
            return;
        }
        let i = self
            .selected()
            .map_or(self.len() - 1, Self::prev_wrapped(self.len()));
        self.select(Some(i));
    }

    fn none(&mut self) {
        self.select(None);
    }

    fn select(&mut self, index: Option<usize>);
    fn selected(&self) -> Option<usize>;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

pub trait PanelState: Debug {
    fn handle_key_events(&mut self, event: KeyEvent, context: AppContext);
    #[allow(unused_variables)]
    fn pass(&mut self, event: KeyEvent, context: AppContext) {}
}

#[derive(Debug)]
pub enum TabState {
    UserList(UserListState),
}

impl TabState {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        match self {
            TabState::UserList(state) => {
                frame.render_stateful_widget(UserList, area, state);
            }
        }
    }

    pub fn update_data(&mut self, user: &User) -> Result<(), UserError> {
        match self {
            TabState::UserList(state) => state.update_data(user),
        }
    }
}

impl PanelState for TabState {
    fn handle_key_events(&mut self, event: KeyEvent, context: AppContext) {
        match self {
            TabState::UserList(state) => state.handle_key_events(event, context),
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

    fn tab(&mut self, index: usize) {
        if index > 0 && index <= self.total_tabs {
            self.current_tab = index - 1
        }
    }
}

impl PanelState for UIState {
    fn handle_key_events(&mut self, event: KeyEvent, context: AppContext) {
        match event.code {
            _ if context.is_override() => self.pass(event, context),
            KeyCode::Tab => self.tab(self.current_tab + 1),
            KeyCode::BackTab => self.tab(self.current_tab - 1),
            KeyCode::Char('1') => self.tab(1),
            KeyCode::Char('2') => self.tab(2),
            _ => self.pass(event, context),
        }
    }
    fn pass(&mut self, event: KeyEvent, context: AppContext) {
        if let Some(tab) = self.get_mut() {
            tab.handle_key_events(event, context);
        }
    }
}
