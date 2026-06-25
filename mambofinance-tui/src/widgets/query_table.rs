use std::fmt::Debug;
use std::marker::PhantomData;

use mambofinance_lib::user::{
    Category, Currency, FieldVariant, FlattenableQuery, Fund, Group, Header, Matrix, Query,
    Transaction, User, UserError,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Cell, Row, StatefulWidget, Table, TableState};

use crate::widgets::PanelState;

const COLUMN_THRESHOLD: usize = 4;

pub struct QueryTable<T> {
    items: Matrix,
    headers: Vec<Header>,
    _marker: PhantomData<T>,
}

impl<T: Fetchable> QueryTable<T>
where
    Query<T>: FlattenableQuery,
{
    pub fn new(query: &Query<T>) -> Self {
        let items = query.flatten();
        let headers = query.headers.clone();
        Self {
            items,
            headers,
            _marker: PhantomData,
        }
    }
}

impl<T: Fetchable> StatefulWidget for QueryTable<T> {
    type State = QueryTableState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let rows: Vec<Row> = self
            .items
            .into_iter()
            .map(|r| Row::new(r.into_iter().map(Cell::from)))
            .collect();

        let (mut headers, mut widths): (Vec<String>, Vec<Constraint>) = self
            .headers
            .iter()
            .map(|(s, w, v)| {
                (
                    s.to_string(),
                    match *v {
                        FieldVariant::Limit => Constraint::Min(*w as u16),
                        _ => Constraint::Length(*w as u16),
                    },
                )
            })
            .unzip();

        if headers.len() <= COLUMN_THRESHOLD {
            headers.push(String::new());
            widths.push(Constraint::Fill(3));
        }

        let header_row = Row::new(headers)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1);

        let table_widget = Table::new(rows, widths)
            .header(header_row)
            .block(Block::bordered().title(" List "))
            .highlight_symbol("> ")
            .row_highlight_style(Style::new().bg(Color::DarkGray))
            .column_spacing(2);

        StatefulWidget::render(table_widget, area, buf, &mut state.state);
    }
}

// region: QueryTableState

#[derive(Debug)]
pub struct QueryTableState<T> {
    pub state: TableState,
    query: Query<T>,
    need_query: bool,
}

impl<T: Fetchable> QueryTableState<T>
where
    Query<T>: FlattenableQuery,
{
    pub fn new(user: &User) -> Result<Self, UserError> {
        let state = TableState::default();
        let query = T::fetch(user)?;

        Ok(QueryTableState {
            state,
            query,
            need_query: true,
        })
    }

    pub fn to_widget(&self) -> QueryTable<T> {
        QueryTable::new(&self.query)
    }

    pub fn update_data(&mut self, user: &User) -> Result<(), UserError> {
        if self.need_query {
            self.query = T::fetch(user)?;
            self.need_query = false;
        }
        Ok(())
    }
}

impl<T: Debug> PanelState for QueryTableState<T> {
    fn next(&mut self) {
        if self.query.is_empty() {
            return;
        }
        let i = self
            .state
            .selected()
            .map_or(0, |cur| self.next_wrapped(self.query.len(), cur));
        self.state.select(Some(i));
    }

    fn prev(&mut self) {
        if self.query.is_empty() {
            return;
        }
        let i = self.state.selected().map_or(self.query.len() - 1, |cur| {
            self.prev_wrapped(self.query.len(), cur)
        });
        self.state.select(Some(i));
    }

    fn none(&mut self) {
        self.state.select(None);
    }

    fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}

// endregion

// region: Fetchable

pub trait Fetchable: Sized {
    fn fetch(user: &User) -> Result<Query<Self>, UserError>;
}

impl Fetchable for Transaction {
    fn fetch(user: &User) -> Result<Query<Self>, UserError> {
        user.transactions()
    }
}

impl Fetchable for Group {
    fn fetch(user: &User) -> Result<Query<Self>, UserError> {
        user.groups()
    }
}

impl Fetchable for Category {
    fn fetch(user: &User) -> Result<Query<Self>, UserError> {
        user.categories()
    }
}

impl Fetchable for Fund {
    fn fetch(user: &User) -> Result<Query<Self>, UserError> {
        user.funds()
    }
}

impl Fetchable for Currency {
    fn fetch(user: &User) -> Result<Query<Self>, UserError> {
        user.currencies()
    }
}

// endregion
