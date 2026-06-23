use ratatui::Frame;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};

use crate::app::App;
use crate::widgets::user_list::UserList;

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    let top_bar = Block::new().title(" MamboFinance ").borders(Borders::ALL);
    frame.render_widget(top_bar, chunks[0]);

    let user_list = UserList::new(&app.user, &mut app.user_list_state);
    frame.render_widget(user_list, chunks[1]);
}
