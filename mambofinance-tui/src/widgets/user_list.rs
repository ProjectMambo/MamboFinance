use std::mem;
use std::str::FromStr;

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
        Actionable, PanelState, UIEvent,
        bottom_bar::{BottomBar, Hintable},
        pop_up::{PopUp, PopUpEvent, PopUpState, Popable},
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
];

pub const TRANSACTION_ENTRY_COUNT: usize = 10;
pub const GROUP_ENTRY_COUNT: usize = 1;
pub const CATEGORY_ENTRY_COUNT: usize = 2;
pub const FUND_ENTRY_COUNT: usize = 1;
pub const CURRENCY_ENTRY_COUNT: usize = 1;

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

        if state.is_pop() {
            let popup_split =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(table_area);
            let overlay_area = popup_split[1];
            Widget::render(Clear, overlay_area, buf);
            StatefulWidget::render(PopUp, overlay_area, buf, &mut state.popup_state);
        }

        if let Some(status) = state.status.as_deref() {
            Widget::render(BottomBar::error(status), v_chunks[1], buf);
        } else {
            Widget::render(BottomBar::new(state.hint()), v_chunks[1], buf);
        }
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
    status: Option<String>,
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

        let popup_state = PopUpState::new("Add");
        Ok(Self {
            sidebar_state,
            table_state,
            popup_state,
            cached_table_state,
            active_index: 0,
            focused: Some(0),
            pop: false,
            status: None,
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

    fn esc(&mut self) {
        self.pop(false)
    }

    fn add(&mut self, user: &User) {
        match self.table_state.to_popup(user) {
            Ok(mut popup) => {
                popup.open();
                self.popup_state = popup;
                self.status = None;
                self.pop(true);
            }
            Err(error) => {
                self.status = Some(format!("Cannot open add form: {error}"));
            }
        }
    }
}

impl Actionable for UserListState {
    fn next(&mut self) {
        if self.is_first() {
            self.table_state.next();
        }
        self.select(self.next_capped());
    }

    fn prev(&mut self) {
        if self.is_last() {
            self.table_state.none();
        }
        self.select(self.prev_capped());
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
    fn handle_key_events(&mut self, event: KeyEvent, context: &AppContext) -> bool {
        if !self.pass(event, context) {
            match event.code {
                KeyCode::Esc => self.esc(),
                KeyCode::Char('a') => self.add(context.user),
                KeyCode::Left | KeyCode::Char('h') => self.prev(),
                KeyCode::Right | KeyCode::Char('l') => self.next(),
                _ => return false,
            }
        }
        true
    }

    fn pass(&mut self, event: KeyEvent, context: &AppContext) -> bool {
        match self.selected() {
            Some(_) if self.is_pop() => self.popup_state.handle_key_events(event, context),
            Some(0) => {
                let handled = self.sidebar_state.handle_key_events(event, context);
                self.update();
                handled
            }
            Some(1) => self.table_state.handle_key_events(event, context),
            _ => false,
        }
    }

    fn handle_ui_events(&mut self, event: UIEvent, context: AppContext) -> bool {
        let Some(popup) = event.try_into_popup() else {
            return false;
        };
        if let PopUpEvent::Add = popup {
            let data = self.popup_state.clone().compile();
            match self.table_state.add(context.user, data) {
                Ok(()) => {
                    self.table_state.need_query();
                    self.status = self.update_data(context.user).err().map(|error| {
                        format!("Record saved, but the view could not refresh: {error}")
                    });
                    self.esc();
                }
                Err(error) => {
                    self.status = Some(format!("Cannot add record: {error}"));
                }
            }
            true
        } else {
            false
        }
    }
}

impl Hintable for UserListState {
    fn hint(&mut self) -> &[(&str, &str)] {
        if self.is_pop() {
            self.popup_state.hint()
        } else {
            HINT_ITEMS
        }
    }
}

impl Popable for UserListState {
    fn is_pop(&self) -> bool {
        self.pop
    }

    fn pop(&mut self, pop: bool) {
        self.pop = pop
    }
}

// endregion

// region: ActiveTable

// wrapper to manage all defined query table state
#[derive(Debug)]
pub enum ActiveTableState {
    Transactions(QueryTableState<Transaction>),
    Groups(QueryTableState<Group>),
    Categories(QueryTableState<Category>),
    Funds(QueryTableState<Fund>),
    Currencies(QueryTableState<Currency>),
    None,
}

// macro for behaviour passing down to wrapped contents
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
        map!(self, w => return w.update_data(user));
        Ok(())
    }

    pub fn need_query(&mut self) {
        map!(self, w => w.need_query());
    }

    // get the popup state of corresponding table
    pub fn to_popup(&self, user: &User) -> Result<PopUpState, String> {
        match self {
            ActiveTableState::Transactions(..) => {
                let groups = user
                    .groups()
                    .map_err(|error| error.to_string())?
                    .to_options();
                let categories = user
                    .categories()
                    .map_err(|error| error.to_string())?
                    .to_options();
                let funds = user
                    .funds()
                    .map_err(|error| error.to_string())?
                    .to_options();
                let currencies = user
                    .currencies()
                    .map_err(|error| error.to_string())?
                    .to_options();
                Ok(transaction_popup(groups, categories, funds, currencies))
            }
            ActiveTableState::Groups(..) => Ok(group_popup()),
            ActiveTableState::Categories(..) => Ok(category_popup()),
            ActiveTableState::Funds(..) => Ok(fund_popup()),
            ActiveTableState::Currencies(..) => Ok(currency_popup()),
            ActiveTableState::None => Err("no active table is selected".to_string()),
        }
    }

    pub fn add(&self, user: &User, data: Vec<Option<String>>) -> Result<(), String> {
        match self {
            ActiveTableState::Transactions(..) => {
                let [name, desc, amt_str, cur, d, m, y, grp, cat, fnd] = data.as_slice() else {
                    return Err(entry_count_error(TRANSACTION_ENTRY_COUNT, data.len()));
                };

                let amount = parse_number::<i64>(amt_str, "Amount")?;
                let day = parse_number::<u8>(d, "Day")?;
                let month = parse_number::<u8>(m, "Month")?;
                let year = parse_number::<u16>(y, "Year")?;

                user.add_transaction(
                    name.as_deref()
                        .filter(|s| !s.is_empty())
                        .unwrap_or("EMPTY TRANSACTION"),
                    desc.as_deref(),
                    (amount, required_value(cur, "Currency")?),
                    (day, month, year),
                    required_value(grp, "Group")?,
                    required_value(cat, "Category")?,
                    required_value(fnd, "Fund")?,
                )
                .map(|_| ())
                .map_err(|error| error.to_string())
            }
            ActiveTableState::Groups(..) => {
                let [name] = data.as_slice() else {
                    return Err(entry_count_error(GROUP_ENTRY_COUNT, data.len()));
                };
                user.add_group(
                    name.as_deref()
                        .filter(|s| !s.is_empty())
                        .unwrap_or("EMPTY GROUP"),
                )
                .map(|_| ())
                .map_err(|error| error.to_string())
            }
            ActiveTableState::Categories(..) => {
                let [name, variant] = data.as_slice() else {
                    return Err(entry_count_error(CATEGORY_ENTRY_COUNT, data.len()));
                };
                let name = name
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .unwrap_or("EMPTY CATEGORY");

                match required_value(variant, "Variant")? {
                    "Paired" => user.add_paired_category(name),
                    "Single" => user.add_category(name),
                    variant => return Err(format!("Unsupported category variant: {variant}")),
                }
                .map(|_| ())
                .map_err(|error| error.to_string())
            }
            ActiveTableState::Funds(..) => {
                let [name] = data.as_slice() else {
                    return Err(entry_count_error(FUND_ENTRY_COUNT, data.len()));
                };
                user.add_fund(
                    name.as_deref()
                        .filter(|s| !s.is_empty())
                        .unwrap_or("EMPTY FUND"),
                )
                .map(|_| ())
                .map_err(|error| error.to_string())
            }
            ActiveTableState::Currencies(..) => {
                let [name] = data.as_slice() else {
                    return Err(entry_count_error(CURRENCY_ENTRY_COUNT, data.len()));
                };
                user.add_currency(
                    name.as_deref()
                        .filter(|s| !s.is_empty())
                        .unwrap_or("EMPTY CURRENCY"),
                )
                .map(|_| ())
                .map_err(|error| error.to_string())
            }
            ActiveTableState::None => Err("no active table is selected".to_string()),
        }
    }
}

fn entry_count_error(expected: usize, actual: usize) -> String {
    format!("Form entry count mismatch: expected {expected}, got {actual}")
}

fn required_value<'a>(value: &'a Option<String>, field: &str) -> Result<&'a str, String> {
    value
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{field} is required"))
}

fn parse_number<T: FromStr>(value: &Option<String>, field: &str) -> Result<T, String> {
    required_value(value, field)?
        .trim()
        .parse()
        .map_err(|_| format!("{field} must be a valid whole number"))
}

impl Actionable for ActiveTableState {
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
    fn handle_key_events(&mut self, event: KeyEvent, context: &AppContext) -> bool {
        map!(self, w => w.handle_key_events(event,context), false)
    }
}

// endregion

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded_user() -> User {
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");
        user.add_category("Food").expect("add_category failed");
        user.add_fund("Cash").expect("add_fund failed");
        user.add_currency("USD").expect("add_currency failed");
        user
    }

    fn transaction_form(amount: &str, day: &str, month: &str) -> Vec<Option<String>> {
        [
            "Lunch", "", amount, "USD", day, month, "2026", "Personal", "Food", "Cash",
        ]
        .into_iter()
        .map(|value| Some(value.to_string()))
        .collect()
    }

    #[test]
    fn add_transaction_rejects_malformed_amount_without_writing() {
        let user = seeded_user();
        let state = ActiveTableState::Transactions(
            QueryTableState::<Transaction>::new(&user).expect("transaction table should build"),
        );

        let result = state.add(&user, transaction_form("not-a-number", "1", "1"));

        assert!(matches!(result, Err(ref error) if error.contains("Amount")));
        assert!(user.transactions().unwrap().rows.is_empty());
    }

    #[test]
    fn add_transaction_surfaces_invalid_date_without_writing() {
        let user = seeded_user();
        let state = ActiveTableState::Transactions(
            QueryTableState::<Transaction>::new(&user).expect("transaction table should build"),
        );

        let result = state.add(&user, transaction_form("100", "0", "1"));

        assert!(matches!(result, Err(ref error) if error.contains("valid day")));
        assert!(user.transactions().unwrap().rows.is_empty());
    }

    #[test]
    fn add_surfaces_database_validation_errors() {
        let user = seeded_user();
        let state = ActiveTableState::Groups(
            QueryTableState::<Group>::new(&user).expect("group table should build"),
        );

        let result = state.add(&user, vec![Some("Personal".to_string())]);

        assert!(matches!(result, Err(ref error) if error.contains("already exists")));
        assert_eq!(user.groups().unwrap().rows.len(), 1);
    }
}
