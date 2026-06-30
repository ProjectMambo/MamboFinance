use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct BottomBar {
    hint: Vec<(String, String)>,
}

impl BottomBar {
    pub fn new(hint: &[(&str, &str)]) -> Self {
        Self {
            hint: hint
                .iter()
                .map(|(h, k)| (h.to_string(), k.to_string()))
                .collect(),
        }
    }
}

impl Widget for BottomBar {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::default().borders(Borders::ALL).title(" Hint ");

        let mut spans = Vec::new();
        let total_items = self.hint.len();

        for (i, (key, val)) in self.hint.into_iter().enumerate() {
            spans.push(Span::styled(
                format!(" {}: ", key),
                Style::default().fg(Color::Cyan).bold(),
            ));
            spans.push(Span::raw(val));

            if i < total_items - 1 {
                spans.push(Span::styled(
                    " |",
                    Style::default().fg(Color::Yellow).bold(),
                ));
            }
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).block(block);
        paragraph.render(area, buf);
    }
}

pub trait Hintable {
    fn hint(&mut self) -> &[(&str, &str)];

    fn empty() -> &'static [(&'static str, &'static str)] {
        &[("", "")]
    }
}
