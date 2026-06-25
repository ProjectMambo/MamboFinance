use std::mem;

use mambofinance_lib::user::{Category, Currency, Fund, Group, Transaction, User, UserError};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::StatefulWidget,
};

use crate::widgets::{
    FocusState, PanelState, query_table::QueryTableState, side_bar::SideBarState,
};

pub const MENU_ITEMS: &[&str] = &[
    "Transactions",
    "Groups",
    "Categories",
    "Funds",
    "Currencies",
];

pub struct UserList {}

impl UserList {
    pub fn new() -> Self {
        Self {}
    }
}

impl StatefulWidget for UserList {
    type State = UserListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let menu_items = Vec::from(MENU_ITEMS);
        let max_sidebar_width = menu_items.iter().map(|s| s.len()).max().unwrap_or(12) + 6;

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(max_sidebar_width as u16),
                Constraint::Min(0),
            ])
            .split(area);

        let sidebar = state.sidebar_state.to_widget();
        StatefulWidget::render(sidebar, chunks[0], buf, &mut state.sidebar_state);

        state.table_state.render(chunks[1], buf);
    }
}

#[derive(Debug)]
pub struct UserListState {
    pub sidebar_state: SideBarState,
    pub table_state: ActiveTableState,
    cached_table_state: Vec<ActiveTableState>,
    active_index: usize,
    focused: Option<usize>,
}

impl UserListState {
    pub fn new(user: &User) -> Result<Self, UserError> {
        let sidebar_state = SideBarState::new(MENU_ITEMS);
        let table_state =
            ActiveTableState::Transactions(QueryTableState::<Transaction>::new(user)?);
        let cached_table_state = vec![
            ActiveTableState::None,
            ActiveTableState::Groups(QueryTableState::<Group>::new(user)?),
            ActiveTableState::Categories(QueryTableState::<Category>::new(user)?),
            ActiveTableState::Funds(QueryTableState::<Fund>::new(user)?),
            ActiveTableState::Currencies(QueryTableState::<Currency>::new(user)?),
        ];
        Ok(Self {
            sidebar_state,
            table_state,
            cached_table_state,
            active_index: 0,
            focused: None,
        })
    }

    pub fn to_widget(&self) -> UserList {
        UserList::new()
    }

    fn update_cached(&mut self, index: usize) {
        if index == self.active_index {
            return;
        };
        let cached = mem::replace(&mut self.cached_table_state[index], ActiveTableState::None);
        let ori_active = mem::replace(&mut self.table_state, cached);
        self.cached_table_state[self.active_index] = ori_active;
        self.active_index = index;
    }

    pub fn update(&mut self) {
        if let Some(selected) = self.sidebar_state.sync() {
            self.update_cached(selected);
        }
    }

    pub fn update_data(&mut self, user: &User) -> Result<(), UserError> {
        self.table_state.update_data(user)?;
        self.cached_table_state
            .iter_mut()
            .try_for_each(|t| t.update_data(user))
    }
}

impl FocusState for UserListState {
    fn focus_next(&mut self) {
        self.focused = match self.focused {
            Some(0) => {
                self.table_state.next();
                Some(1)
            }
            Some(1) => Some(1),
            Some(_) => {
                self.table_state.none();
                Some(0)
            }
            None => match self.sidebar_state.selected() {
                Some(_) => {
                    self.table_state.next();
                    Some(1)
                }
                None => {
                    self.sidebar_state.next();
                    Some(0)
                }
            },
        }
    }

    fn focus_prev(&mut self) {
        self.focused = match self.focused {
            Some(0) => Some(0),
            Some(_) => {
                self.table_state.none();
                Some(0)
            }
            None => Some(0),
        }
    }

    fn next(&mut self) {
        match self.focused {
            Some(0) => {
                self.sidebar_state.next();
                self.update()
            }
            Some(_) => self.table_state.next(),
            None => {
                self.sidebar_state.next();
                self.update()
            }
        };
    }

    fn prev(&mut self) {
        match self.focused {
            Some(0) => {
                self.sidebar_state.prev();
                self.update()
            }
            Some(_) => self.table_state.prev(),
            None => {
                self.sidebar_state.prev();
                self.update()
            }
        };
    }
}

#[derive(Debug)]
pub enum ActiveTableState {
    Transactions(QueryTableState<Transaction>),
    Groups(QueryTableState<Group>),
    Categories(QueryTableState<Category>),
    Funds(QueryTableState<Fund>),
    Currencies(QueryTableState<Currency>),
    None,
}

macro_rules! map {
    ($self:expr, $wrapper:ident => $action:expr) => {
        match $self {
            ActiveTableState::Transactions($wrapper) => $action,
            ActiveTableState::Groups($wrapper) => $action,
            ActiveTableState::Categories($wrapper) => $action,
            ActiveTableState::Funds($wrapper) => $action,
            ActiveTableState::Currencies($wrapper) => $action,
            ActiveTableState::None => {}
        }
    };
}

impl ActiveTableState {
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        if let ActiveTableState::None = self {
            return;
        }

        map!(self, w => {
            let widget = w.to_widget();
            widget.render(area, buf, w);
        });
    }

    pub fn update_data(&mut self, user: &User) -> Result<(), UserError> {
        map!(self, w => {return w.update_data(user);});
        Ok(())
    }
}

impl PanelState for ActiveTableState {
    fn next(&mut self) {
        map!(self, w => w.next())
    }

    fn prev(&mut self) {
        map!(self, w => w.prev())
    }

    fn none(&mut self) {
        map!(self, w => w.none())
    }

    fn selected(&self) -> Option<usize> {
        map!(self, w => {return w.selected();});
        None
    }
}
