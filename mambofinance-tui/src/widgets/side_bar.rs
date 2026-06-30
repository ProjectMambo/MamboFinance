use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

use crate::{
    app::AppContext,
    widgets::{Actionable, PanelState},
};

pub struct SideBar;

impl StatefulWidget for SideBar {
    type State = SideBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items: Vec<ListItem> = state
            .tabs
            .iter()
            .map(|tab_name| ListItem::new(format!(" {tab_name}")))
            .collect();

        let sidebar_widget = List::new(items)
            .block(Block::default().title(" Menu ").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">");

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
            shown: Some(0),
        }
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

impl Actionable for SideBarState {
    fn select(&mut self, index: Option<usize>) {
        self.state.select(index);
    }

    fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    fn len(&self) -> usize {
        self.tabs.len()
    }
}

impl PanelState for SideBarState {
    fn handle_key_events(
        &mut self,
        event: KeyEvent,
        #[allow(unused_variables)] context: AppContext,
    ) {
        match event.code {
            _ if context.is_override() => self.pass(event, context),
            KeyCode::Up | KeyCode::Char('k') => self.prev(),
            KeyCode::Down | KeyCode::Char('j') => self.next(),
            _ => self.pass(event, context),
        }
    }
}
