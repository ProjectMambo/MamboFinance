use std::fmt::Debug;

use crossterm::event::{KeyCode, KeyEvent};
use mambofinance_lib::user::{User, UserError};
use ratatui::{Frame, layout::Rect};

use crate::{
    app::AppContext,
    widgets::{
        pop_up::PopUpEvent,
        user_list::{UserList, UserListState},
    },
};

pub mod bottom_bar;
pub mod pop_up;
pub mod query_table;
pub mod side_bar;
pub mod user_list;

// trait for easier nesting widgets allowing states to control child widgets
pub trait Actionable: Debug {
    // filter last to None, fallback to first
    fn next_wrapped(&self) -> Option<usize> {
        self.selected()
            .filter(|_| !self.is_last())
            .map(|i| i + 1)
            .or(self.first())
    }

    // filter first to None, fallback to last
    fn prev_wrapped(&self) -> Option<usize> {
        self.selected()
            .filter(|_| !self.is_first())
            .map(|i| i - 1)
            .or(self.last())
    }

    // fallback to first, filter last to None, fallback to last
    fn next_capped(&self) -> Option<usize> {
        self.selected()
            .map(|i| i + 1)
            .or(Some(0))
            .filter(|&i| i < self.len())
            .or(self.last())
    }

    // fallback to last, filter first to None, fallback to first
    fn prev_capped(&self) -> Option<usize> {
        self.selected()
            .map(|i| i.wrapping_sub(1))
            .or(self.last())
            .filter(|&i| i < self.len())
            .or(Some(0))
    }

    // default next is wrapped
    fn next(&mut self) {
        if !self.is_empty() {
            self.select(self.next_wrapped());
        }
    }

    // default prev is wrapped
    fn prev(&mut self) {
        if !self.is_empty() {
            self.select(self.prev_wrapped());
        }
    }

    // deselect
    fn none(&mut self) {
        self.select(None);
    }

    fn select(&mut self, index: Option<usize>);
    fn selected(&self) -> Option<usize>;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;

    fn first(&self) -> Option<usize> {
        Some(0)
    }
    fn last(&self) -> Option<usize> {
        Some(self.len().saturating_sub(1))
    }
    fn is_first(&self) -> bool {
        self.selected() == self.first()
    }
    fn is_last(&self) -> bool {
        self.selected() == self.last()
    }
}

pub trait Focusable {
    fn focus(&mut self) {
        self.set_focus(true);
    }
    fn unfocus(&mut self) {
        self.set_focus(false);
    }

    fn is_focus(&self) -> bool;
    fn set_focus(&mut self, focus: bool);
}

pub trait Gettable<T> {
    fn get(&self) -> Option<&T>;
    fn get_mut(&mut self) -> Option<&mut T>;
}

// trait for easier events handling for nested childs
pub trait PanelState: Debug {
    // widget specific handling
    fn handle_key_events(&mut self, event: KeyEvent, context: &AppContext) -> bool;
    #[allow(unused_variables)]
    // pass down keyevents to child widgets
    fn pass(&mut self, event: KeyEvent, context: &AppContext) -> bool {
        false
    }
    #[allow(unused_variables)]
    // handle custom events, consume context
    fn handle_ui_events(&mut self, event: UIEvent, context: AppContext) -> bool {
        false
    }
}

// wrapper to manage all tabs
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
    fn handle_key_events(&mut self, event: KeyEvent, context: &AppContext) -> bool {
        match self {
            TabState::UserList(state) => state.handle_key_events(event, context),
        }
    }
    fn handle_ui_events(&mut self, event: UIEvent, context: AppContext) -> bool {
        match self {
            TabState::UserList(state) => state.handle_ui_events(event, context),
        }
    }
}

// main state to handle all widgets state
#[derive(Debug)]
pub struct UIState {
    current_tab: usize,
    pub tabs: Vec<TabState>,
}

impl UIState {
    pub fn new(tabs: Vec<TabState>) -> Self {
        UIState {
            current_tab: 0,
            tabs,
        }
    }
}

impl Gettable<TabState> for UIState {
    fn get(&self) -> Option<&TabState> {
        self.tabs.get(self.current_tab)
    }
    fn get_mut(&mut self) -> Option<&mut TabState> {
        self.tabs.get_mut(self.current_tab)
    }
}

impl Actionable for UIState {
    fn next(&mut self) {
        if !self.is_empty() {
            self.select(self.next_capped());
        }
    }
    fn prev(&mut self) {
        if !self.is_empty() {
            self.select(self.prev_capped());
        }
    }
    fn select(&mut self, index: Option<usize>) {
        self.current_tab = index.unwrap_or(0)
    }
    fn selected(&self) -> Option<usize> {
        Some(self.current_tab)
    }
    fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }
    fn len(&self) -> usize {
        self.tabs.len()
    }
}

impl PanelState for UIState {
    fn handle_key_events(&mut self, event: KeyEvent, context: &AppContext) -> bool {
        if !self.pass(event, context) {
            match event.code {
                KeyCode::Tab => self.next(),
                KeyCode::BackTab => self.prev(),
                KeyCode::Char('1') => self.select(Some(1)),
                KeyCode::Char('2') => self.select(Some(2)),
                _ => return false,
            }
            true
        } else {
            false
        }
    }
    fn pass(&mut self, event: KeyEvent, context: &AppContext) -> bool {
        if let Some(tab) = self.get_mut() {
            tab.handle_key_events(event, context)
        } else {
            false
        }
    }
    fn handle_ui_events(&mut self, event: UIEvent, context: AppContext) -> bool {
        if let Some(tab) = self.get_mut() {
            tab.handle_ui_events(event, context)
        } else {
            false
        }
    }
}

// wrapper to manage all custom events
pub enum UIEvent {
    PopUp(PopUpEvent),
    Placeholder(PopUpEvent),
}

impl UIEvent {
    pub fn try_into_popup(self) -> Option<PopUpEvent> {
        match self {
            UIEvent::PopUp(e) => Some(e),
            _ => None,
        }
    }
}
