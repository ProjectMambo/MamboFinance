use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

use crate::widgets::PanelState;

pub struct SideBar {
    tabs: Vec<String>,
}

impl SideBar {
    pub fn new(tabs: Vec<String>) -> Self {
        Self { tabs }
    }
}

impl StatefulWidget for SideBar {
    type State = SideBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items: Vec<ListItem> = self
            .tabs
            .iter()
            .map(|tab_name| ListItem::new(tab_name.as_str()))
            .collect();

        let sidebar_widget = List::new(items)
            .block(Block::default().title(" Menu ").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        StatefulWidget::render(sidebar_widget, area, buf, &mut state.state);
    }
}

#[derive(Debug)]
pub struct SideBarState {
    pub state: ListState,
    tabs: Vec<String>,
    pub shown: Option<usize>,
}

impl SideBarState {
    pub fn new(tabs: &[&str]) -> Self {
        let state = ListState::default();
        Self {
            state,
            tabs: tabs.iter().map(|t| t.to_string()).collect(),
            shown: None,
        }
    }

    pub fn to_widget(&self) -> SideBar {
        SideBar::new(self.tabs.clone())
    }

    pub fn sync(&mut self) -> Option<usize> {
        if self.shown != self.selected() {
            self.shown = self.selected();
            self.shown
        } else {
            None
        }
    }
}

impl PanelState for SideBarState {
    fn next(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        let i = self
            .state
            .selected()
            .map_or(0, |cur| self.next_wrapped(self.tabs.len(), cur));
        self.state.select(Some(i));
    }

    fn prev(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        let i = self.state.selected().map_or(self.tabs.len() - 1, |cur| {
            self.prev_wrapped(self.tabs.len(), cur)
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
