use std::mem;

use crossterm::event::{KeyCode, KeyEvent};
use mambofinance_lib::user::{Category, Currency, Fund, Group, Transaction, User, UserError};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Clear, StatefulWidget, Widget},
};

use crate::{
    app::AppContext,
    widgets::{
        Actionable, PanelState,
        bottom_bar::{BottomBar, Hintable},
        pop_up::{PopUp, PopUpState},
        query_table::{QueryTable, QueryTableState},
        side_bar::{SideBar, SideBarState},
    },
};

// region: Config

pub const MENU_ITEMS: &[&str] = &[
    "Transactions",
    "Groups",
    "Categories",
    "Funds",
    "Currencies",
];

pub const HINT_ITEMS: &[(&str, &str)] = &[
    ("Quit", "ctrl c"),
    ("Navigate", "h/j/k/l | ←/↓/↑/→"),
    ("Add", "a"),
    ("Delete", "d"),
    ("Edit", "e"),
    ("Sort", "s"),
    ("Filter", "f"),
];

fn transaction_popup(
    groups: Vec<String>,
    categories: Vec<String>,
    funds: Vec<String>,
    currencies: Vec<String>,
) -> PopUpState {
    let mut state = PopUpState::new("Add Transaction");
    state
        .row()
        .input("Name")
        .row()
        .input("Description")
        .row()
        .input("Amount")
        .vertical("Currency", Some(currencies))
        .row()
        .input("Day")
        .input("Month")
        .input("Year")
        .row()
        .horizontal("Group", Some(groups))
        .row()
        .horizontal("Category", Some(categories))
        .row()
        .horizontal("Fund", Some(funds))
        .complete();
    state
}

fn group_popup() -> PopUpState {
    let mut state = PopUpState::new("Add Group");
    state.input("Name").complete();
    state
}

fn category_popup() -> PopUpState {
    let mut state = PopUpState::new("Add Category");
    state
        .row()
        .input("Name")
        .row()
        .horizontal("Variant", Some(vec!["Single", "Paired"]))
        .complete();
    state
}

fn fund_popup() -> PopUpState {
    let mut state = PopUpState::new("Add Fund");
    state.input("Name").complete();
    state
}

fn currency_popup() -> PopUpState {
    let mut state = PopUpState::new("Add Currency");
    state.input("Name").complete();
    state
}

// endregion

// region: UserList

pub struct UserList;

impl StatefulWidget for UserList {
    type State = UserListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let sidebar_w = MENU_ITEMS.iter().map(|s| s.len()).max().unwrap_or(12) as u16 + 6;

        let v_chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(area);
        let h_chunks = Layout::horizontal([Constraint::Length(sidebar_w), Constraint::Min(0)])
            .split(v_chunks[0]);
        let table_area = h_chunks[1];

        StatefulWidget::render(SideBar, h_chunks[0], buf, &mut state.sidebar_state);
        state.table_state.render(table_area, buf);

        if state.pop {
            let popup_split =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(table_area);
            let overlay_area = popup_split[1];
            Widget::render(Clear, overlay_area, buf);
            StatefulWidget::render(PopUp, overlay_area, buf, &mut state.popup_state);
        }

        Widget::render(BottomBar::new(state.hint()), v_chunks[1], buf);
    }
}

#[derive(Debug)]
pub struct UserListState {
    pub sidebar_state: SideBarState,
    pub table_state: ActiveTableState,
    pub popup_state: PopUpState,
    cached_table_state: Vec<ActiveTableState>,
    active_index: usize,
    focused: Option<usize>,
    pop: bool,
}

impl UserListState {
    pub fn new(user: &User) -> Result<Self, UserError> {
        let mut sidebar_state = SideBarState::new(MENU_ITEMS);
        sidebar_state.next();

        let table_state =
            ActiveTableState::Transactions(QueryTableState::<Transaction>::new(user)?);
        let cached_table_state = vec![
            ActiveTableState::None,
            ActiveTableState::Groups(QueryTableState::<Group>::new(user)?),
            ActiveTableState::Categories(QueryTableState::<Category>::new(user)?),
            ActiveTableState::Funds(QueryTableState::<Fund>::new(user)?),
            ActiveTableState::Currencies(QueryTableState::<Currency>::new(user)?),
        ];

        let popup_state = table_state.to_popup(user);
        Ok(Self {
            sidebar_state,
            table_state,
            popup_state,
            cached_table_state,
            active_index: 0,
            focused: Some(0),
            pop: false,
        })
    }

    fn update_cached(&mut self, index: usize) {
        if index == self.active_index {
            return;
        }
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

    fn esc(&mut self, context: AppContext) {
        if self.pop {
            context.input(false);
            self.prev();
            self.pop = false;
        }
    }

    fn add(&mut self, context: AppContext) {
        context.input(true);
        self.popup_state = self.table_state.to_popup(context.user);
        self.pop = true;
        self.next();
        self.popup_state.next();
    }
}

impl Actionable for UserListState {
    fn next(&mut self) {
        let index = match self.selected() {
            Some(0) if !self.pop => {
                self.table_state.next();
                Some(1)
            }
            Some(_) => Some(1),
            None => Some(0),
        };
        self.select(index);
    }

    fn prev(&mut self) {
        if self.selected().is_some() && !self.pop {
            self.table_state.none();
        }
        self.select(Some(0));
    }

    fn select(&mut self, index: Option<usize>) {
        self.focused = index;
    }

    fn selected(&self) -> Option<usize> {
        self.focused
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn len(&self) -> usize {
        2
    }
}

impl PanelState for UserListState {
    fn handle_key_events(&mut self, event: KeyEvent, context: AppContext) {
        match event.code {
            KeyCode::Esc if self.pop => self.esc(context),
            _ if context.is_override() => self.pass(event, context),
            KeyCode::Left | KeyCode::Char('h') => self.prev(),
            KeyCode::Right | KeyCode::Char('l') => self.next(),
            KeyCode::Char('a') => self.add(context),
            _ => self.pass(event, context),
        }
    }

    fn pass(&mut self, event: KeyEvent, context: AppContext) {
        match self.focused {
            Some(0) | None => {
                self.sidebar_state.handle_key_events(event, context);
                self.update();
            }
            Some(1) => {
                if self.pop {
                    self.popup_state.handle_key_events(event, context);
                } else {
                    self.table_state.handle_key_events(event, context);
                }
            }
            _ => {}
        }
    }
}

impl Hintable for UserListState {
    fn hint(&mut self) -> &[(&str, &str)] {
        if self.pop {
            self.popup_state.hint()
        } else {
            HINT_ITEMS
        }
    }
}

// endregion

// region: ActiveTable

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
        map!($self, $wrapper => $action, {})
    };
    ($self:expr, $wrapper:ident => $action:expr, $return:expr) => {
        match $self {
            ActiveTableState::None => { $return },
            ActiveTableState::Transactions($wrapper) => $action,
            ActiveTableState::Groups($wrapper) => $action,
            ActiveTableState::Categories($wrapper) => $action,
            ActiveTableState::Funds($wrapper) => $action,
            ActiveTableState::Currencies($wrapper) => $action,
        }
    };
}

impl ActiveTableState {
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        if let ActiveTableState::None = self {
            return;
        }
        map!(self, w => {
            let widget = QueryTable::new();
            widget.render(area, buf, w);
        });
    }

    pub fn update_data(&mut self, user: &User) -> Result<(), UserError> {
        map!(self, w => { return w.update_data(user); });
        Ok(())
    }

    pub fn to_popup(&self, user: &User) -> PopUpState {
        match self {
            ActiveTableState::Transactions(..) => {
                let groups = user.groups().unwrap().to_options();
                let categories = user.categories().unwrap().to_options();
                let funds = user.funds().unwrap().to_options();
                let currencies = user.currencies().unwrap().to_options();
                transaction_popup(groups, categories, funds, currencies)
            }
            ActiveTableState::Groups(..) => group_popup(),
            ActiveTableState::Categories(..) => category_popup(),
            ActiveTableState::Funds(..) => fund_popup(),
            ActiveTableState::Currencies(..) => currency_popup(),
            ActiveTableState::None => unreachable!("U broke the app"),
        }
    }
}

impl Actionable for ActiveTableState {
    fn next(&mut self) {
        map!(self, w => w.next());
    }
    fn prev(&mut self) {
        map!(self, w => w.prev());
    }
    fn none(&mut self) {
        map!(self, w => w.none());
    }
    fn select(&mut self, index: Option<usize>) {
        map!(self, w => w.select(index));
    }
    fn selected(&self) -> Option<usize> {
        map!(self, w => w.selected(), None)
    }
    fn is_empty(&self) -> bool {
        map!(self, w => w.is_empty(), true)
    }
    fn len(&self) -> usize {
        map!(self, w => w.len(), 0)
    }
}

impl PanelState for ActiveTableState {
    fn handle_key_events(&mut self, event: KeyEvent, context: AppContext) {
        map!(self, w => w.handle_key_events(event,context))
    }
}

// endregion
