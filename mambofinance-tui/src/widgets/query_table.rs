use mambofinance_lib::user::{
    Category, Currency, FieldVariant, FlattenableQuery, Fund, Group, Query, Refreshable,
    Transaction, User, UserError,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Cell, Row, StatefulWidget, Table, TableState};

use crate::widgets::Navigable;

const COLUMN_THRESHOLD: usize = 4;

pub struct QueryTable<'a, T> {
    user: &'a User,
    pub query: Query<'a, T>,
}

impl<'a, T: Fetchable<'a>> QueryTable<'a, T> {
    pub fn new(user: &'a User) -> Result<Self, UserError> {
        let query = T::fetch(user)?;

        Ok(QueryTable { user, query })
    }
}

impl<'a, T: Fetchable<'a>> Navigable for QueryTable<'a, T> {
    type State = TableState;

    fn next(&self, state: &mut Self::State) {
        if self.query.is_empty() {
            return;
        }
        let i = state
            .selected()
            .map_or(0, Self::next_wrapped(self.query.len()));
        state.select(Some(i));
    }

    fn previous(&self, state: &mut Self::State) {
        if self.query.is_empty() {
            return;
        }
        let i = state
            .selected()
            .map_or(0, Self::previous_wrapped(self.query.len()));
        state.select(Some(i));
    }
}

impl<'a, T: Fetchable<'a>> StatefulWidget for QueryTable<'a, T>
where
    Query<'a, T>: FlattenableQuery,
{
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let rows: Vec<Row> = self
            .query
            .flatten()
            .into_iter()
            .map(|r| Row::new(r.into_iter().map(Cell::from)))
            .collect();

        let (mut headers, mut widths): (Vec<String>, Vec<Constraint>) = self
            .query
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
            .highlight_symbol(">> ")
            .row_highlight_style(Style::new().bg(Color::DarkGray))
            .column_spacing(2);

        StatefulWidget::render(table_widget, area, buf, state);
    }
}

pub trait Fetchable<'a>: Sized {
    fn fetch(user: &'a User) -> Result<Query<'a, Self>, UserError>;
}

impl<'a> Fetchable<'a> for Transaction {
    fn fetch(user: &'a User) -> Result<Query<'a, Self>, UserError> {
        user.transactions()
    }
}

impl<'a> Fetchable<'a> for Group {
    fn fetch(user: &'a User) -> Result<Query<'a, Self>, UserError> {
        user.groups()
    }
}

impl<'a> Fetchable<'a> for Category {
    fn fetch(user: &'a User) -> Result<Query<'a, Self>, UserError> {
        user.categories()
    }
}

impl<'a> Fetchable<'a> for Fund {
    fn fetch(user: &'a User) -> Result<Query<'a, Self>, UserError> {
        user.funds()
    }
}

impl<'a> Fetchable<'a> for Currency {
    fn fetch(user: &'a User) -> Result<Query<'a, Self>, UserError> {
        user.currencies()
    }
}
