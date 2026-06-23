use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

use crate::widgets::Navigable;

pub struct SideBar {
    tabs: Vec<String>,
}

impl SideBar {
    pub fn new(tabs: Vec<&str>) -> Self {
        Self {
            tabs: tabs.into_iter().map(|t| t.to_string()).collect(),
        }
    }
}

impl Navigable for SideBar {
    type State = ListState;

    fn next(&self, state: &mut Self::State) {
        if self.tabs.is_empty() {
            return;
        }
        let i = state
            .selected()
            .map_or(0, Self::next_wrapped(self.tabs.len()));
        state.select(Some(i));
    }

    fn previous(&self, state: &mut Self::State) {
        if self.tabs.is_empty() {
            return;
        }
        let i = state
            .selected()
            .map_or(0, Self::previous_wrapped(self.tabs.len()));
        state.select(Some(i));
    }
}

impl StatefulWidget for SideBar {
    type State = ListState;

    fn render(self, area: ratatui::prelude::Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items: Vec<ListItem> = self.tabs.into_iter().map(ListItem::new).collect();

        let sidebar_widget = List::new(items)
            .block(Block::default().title(" Menu ").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        StatefulWidget::render(sidebar_widget, area, buf, state);
    }
}
