use ratatui::Frame;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    let top_bar = Block::new().title(" MamboFinance ").borders(Borders::ALL);
    frame.render_widget(top_bar, chunks[0]);

    if let Some(tab) = app.ui_state.get_mut() {
        tab.render(frame, chunks[1]);
    }
}
