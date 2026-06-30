use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use crate::{
    app::AppContext,
    widgets::{Actionable, PanelState, bottom_bar::Hintable},
};

pub const HINT_HORI: &[(&str, &str)] = &[
    ("Quit", "ctrl c"),
    ("Navigate", "tab/btab"),
    ("Select", "←/→"),
    ("Confirm", "enter"),
    ("Cancel", "esc"),
];

pub const HINT_VER: &[(&str, &str)] = &[
    ("Quit", "ctrl c"),
    ("Navigate", "tab/btab"),
    ("Select", "↓/↑"),
    ("Confirm", "enter"),
    ("Cancel", "esc"),
];

pub const HINT_INPUT: &[(&str, &str)] = &[
    ("Quit", "ctrl c"),
    ("Navigate", "tab/btab"),
    ("Input", "type"),
    ("Delete", "backspace"),
    ("Clear", "ctrl backspace"),
    ("Confirm", "enter"),
    ("Cancel", "esc"),
];

// region: Config

#[derive(Debug, Clone)]
pub enum EntryKind {
    HorizontalOption,
    VerticalOption,
    Input,
}

#[derive(Debug, Clone)]
pub struct EntryConfig {
    pub header: String,
    pub kind: EntryKind,
}

impl EntryConfig {
    pub fn horizontal(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            kind: EntryKind::HorizontalOption,
        }
    }

    pub fn vertical(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            kind: EntryKind::VerticalOption,
        }
    }

    pub fn input(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            kind: EntryKind::Input,
        }
    }
}

// endregion

// region: Entry

#[derive(Debug, Clone)]
pub struct EntryState {
    pub config: EntryConfig,
    pub options: Vec<String>,
    selected: Option<usize>,
    value: String,
    focused: bool,
}

impl EntryState {
    pub fn new(config: EntryConfig) -> Self {
        Self {
            config,
            options: Vec::new(),
            selected: Some(0),
            value: String::new(),
            focused: false,
        }
    }

    pub fn with_options(mut self, options: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.options = options.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn unfocus(&mut self) {
        self.focused = false;
    }

    fn border_style(&self) -> Style {
        if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        }
    }

    fn render_horizontal(&self, area: Rect, buf: &mut Buffer) {
        let len = self.options.len();
        if len == 0 {
            return;
        }
        let index = self.selected.unwrap_or(0);
        let option = &self.options[index];
        let formatted = if self.selected == Some(len - 1) {
            format!("← {option}  ")
        } else if self.selected.unwrap_or(0) == 0 {
            format!("  {option} →")
        } else {
            format!("← {option} →")
        };

        let paragraph = Paragraph::new(formatted)
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .title(format!(" {} ", self.config.header))
                    .border_style(self.border_style()),
            );
        Widget::render(paragraph, area, buf);
    }

    fn render_vertical(&self, area: Rect, buf: &mut Buffer) {
        let len = self.options.len();
        if len == 0 {
            return;
        }
        let index = self.selected.unwrap_or(0);
        let option = &self.options[index];
        let formatted = if self.selected == Some(len - 1) {
            format!("  {option} ↑")
        } else if self.selected.unwrap_or(0) == 0 {
            format!("↓ {option}  ")
        } else {
            format!("↓ {option} ↑")
        };

        let paragraph = Paragraph::new(formatted)
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .title(format!(" {} ", self.config.header))
                    .border_style(self.border_style()),
            );
        Widget::render(paragraph, area, buf);
    }

    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        let paragraph = Paragraph::new(self.value.clone()).block(
            Block::bordered()
                .title(format!(" {} ", self.config.header))
                .border_style(self.border_style()),
        );
        Widget::render(paragraph, area, buf);
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        match self.config.kind {
            EntryKind::HorizontalOption => self.render_horizontal(area, buf),
            EntryKind::VerticalOption => self.render_vertical(area, buf),
            EntryKind::Input => self.render_input(area, buf),
        }
    }
}

impl Actionable for EntryState {
    fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    fn selected(&self) -> Option<usize> {
        self.selected
    }

    fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    fn len(&self) -> usize {
        self.options.len()
    }

    fn next(&mut self) {
        if self.is_empty() {
            return;
        }
        let i = self.selected().map_or(0, Self::next_capped(self.len()));
        self.select(Some(i));
    }

    fn prev(&mut self) {
        if self.is_empty() {
            return;
        }
        let i = self.selected().map_or(0, Self::prev_capped());
        self.select(Some(i));
    }
}

impl PanelState for EntryState {
    fn handle_key_events(
        &mut self,
        event: KeyEvent,
        #[allow(unused_variables)] context: AppContext,
    ) {
        match self.config.kind {
            EntryKind::HorizontalOption => match event.code {
                KeyCode::Char(c) => self.value.push(c),
                KeyCode::Backspace => {
                    self.value.clear();
                }
                KeyCode::Left => self.prev(),
                KeyCode::Right => self.next(),
                _ => self.pass(event, context),
            },
            EntryKind::VerticalOption => match event.code {
                KeyCode::Char(c) => self.value.push(c),
                KeyCode::Backspace => {
                    self.value.clear();
                }
                KeyCode::Up => self.prev(),
                KeyCode::Down => self.next(),
                _ => self.pass(event, context),
            },
            EntryKind::Input => match event.code {
                KeyCode::Char(c) => self.value.push(c),
                KeyCode::Backspace if event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.value.clear();
                }
                KeyCode::Backspace => {
                    self.value.pop();
                }
                _ => self.pass(event, context),
            },
        }
    }
}

impl Hintable for EntryState {
    fn hint(&mut self) -> &[(&str, &str)] {
        match self.config.kind {
            EntryKind::HorizontalOption => HINT_HORI,
            EntryKind::VerticalOption => HINT_VER,
            EntryKind::Input => HINT_INPUT,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RowState {
    pub entries: Vec<EntryState>,
    focused: Option<usize>,
}

impl RowState {
    pub fn new(entries: Vec<EntryState>) -> Self {
        Self {
            entries,
            focused: None,
        }
    }

    pub fn focus_last(&mut self) {
        let last = self.entries.len().saturating_sub(1);
        self.select(Some(last));
    }

    pub fn unfocus_all(&mut self) {
        if let Some(i) = self.focused
            && let Some(e) = self.entries.get_mut(i)
        {
            e.unfocus();
        }
        self.focused = None;
    }

    pub fn next_entry(&mut self) -> bool {
        let next = self.focused.map_or(0, |i| i + 1);
        if next < self.len() {
            self.select(Some(next));
            true
        } else {
            false
        }
    }

    pub fn prev_entry(&mut self) -> bool {
        match self.focused {
            Some(0) | None => false,
            Some(i) => {
                self.select(Some(i - 1));
                true
            }
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let constraints = vec![Constraint::Fill(1); self.entries.len()];
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        for (i, entry) in self.entries.iter().enumerate() {
            entry.render(chunks[i], buf);
        }
    }
}

impl Actionable for RowState {
    fn select(&mut self, index: Option<usize>) {
        if let Some(i) = self.selected()
            && let Some(e) = self.entries.get_mut(i)
        {
            e.unfocus();
        }
        self.focused = index;
        if let Some(i) = self.selected()
            && let Some(e) = self.entries.get_mut(i)
        {
            e.focus();
        }
    }

    fn selected(&self) -> Option<usize> {
        self.focused
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl PanelState for RowState {
    fn handle_key_events(&mut self, event: KeyEvent, context: AppContext) {
        self.pass(event, context);
    }

    fn pass(&mut self, event: KeyEvent, context: AppContext) {
        if let Some(i) = self.selected()
            && let Some(entry) = self.entries.get_mut(i)
        {
            entry.handle_key_events(event, context);
        }
    }
}

impl Hintable for RowState {
    fn hint(&mut self) -> &[(&str, &str)] {
        if let Some(i) = self.selected()
            && let Some(e) = self.entries.get_mut(i)
        {
            return e.hint();
        }
        Self::empty()
    }
}

// endregion

// region: Row

#[derive(Debug, Clone)]
pub enum PopUpRow {
    Single(EntryState),
    Multi(RowState),
    Inter(Vec<EntryState>),
}

impl Default for PopUpRow {
    fn default() -> Self {
        PopUpRow::Inter(Vec::new())
    }
}

impl PopUpRow {
    pub fn inter() -> Self {
        PopUpRow::Inter(Vec::new())
    }

    pub fn complete(self) -> Self {
        if let PopUpRow::Inter(mut states) = self {
            match states.len() {
                0 => unreachable!("Empty intermediate shouldn't be possible"),
                1 => {
                    return PopUpRow::Single(
                        states.pop().expect("Intermediate should have 1 element"),
                    );
                }
                _ => {
                    return PopUpRow::Multi(RowState::new(states));
                }
            }
        }
        self
    }

    pub fn focus(&mut self) {
        match self {
            PopUpRow::Single(e) => e.focus(),
            PopUpRow::Multi(r) => {
                if r.selected().is_none() {
                    r.select(Some(0));
                }
            }
            _ => unreachable!("This is an intermediate"),
        }
    }

    pub fn focus_last(&mut self) {
        match self {
            PopUpRow::Single(e) => e.focus(),
            PopUpRow::Multi(r) => r.focus_last(),
            _ => unreachable!("This is an intermediate"),
        }
    }

    pub fn unfocus(&mut self) {
        match self {
            PopUpRow::Single(e) => e.unfocus(),
            PopUpRow::Multi(r) => r.unfocus_all(),
            _ => unreachable!("This is an intermediate"),
        }
    }

    pub fn next_inner(&mut self) -> bool {
        match self {
            PopUpRow::Single(_) => false,
            PopUpRow::Multi(r) => r.next_entry(),
            _ => unreachable!("This is an intermediate"),
        }
    }

    pub fn prev_inner(&mut self) -> bool {
        match self {
            PopUpRow::Single(_) => false,
            PopUpRow::Multi(r) => r.prev_entry(),
            _ => unreachable!("This is an intermediate"),
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        match self {
            PopUpRow::Single(e) => e.render(area, buf),
            PopUpRow::Multi(r) => r.render(area, buf),
            _ => unreachable!("This is an intermediate"),
        }
    }
}

impl PanelState for PopUpRow {
    fn handle_key_events(&mut self, event: KeyEvent, context: AppContext) {
        self.pass(event, context);
    }

    fn pass(&mut self, event: KeyEvent, context: AppContext) {
        match self {
            PopUpRow::Single(e) => e.handle_key_events(event, context),
            PopUpRow::Multi(r) => r.handle_key_events(event, context),
            _ => unreachable!("This is an intermediate"),
        }
    }
}

impl Hintable for PopUpRow {
    fn hint(&mut self) -> &[(&str, &str)] {
        match self {
            PopUpRow::Single(e) => e.hint(),
            PopUpRow::Multi(r) => r.hint(),
            _ => unreachable!("This is an intermediate"),
        }
    }
}

// endregion

// region: PopUp

pub struct PopUp;

impl StatefulWidget for PopUp {
    type State = PopUpState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered().title(format!(" {} ", state.title));
        let inner_area = block.inner(area);
        Widget::render(block, area, buf);

        let constraints = vec![Constraint::Length(3); state.rows.len()];
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner_area);

        for (i, row) in state.rows.iter().enumerate() {
            row.render(chunks[i], buf);
        }
    }
}

#[derive(Debug)]
pub struct PopUpState {
    pub title: String,
    pub rows: Vec<PopUpRow>,
    focused: Option<usize>,
}

impl PopUpState {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            rows: Vec::new(),
            focused: None,
        }
    }

    pub fn row(&mut self) -> &mut Self {
        self.complete();
        self.rows.push(PopUpRow::inter());
        self
    }

    pub fn complete(&mut self) -> &mut Self {
        if let Some(PopUpRow::Inter(_)) = self.rows.last()
            && let Some(row) = self.rows.last_mut()
        {
            *row = std::mem::take(row).complete();
        }
        self
    }

    pub fn input(&mut self, header: impl Into<String>) -> &mut Self {
        if let PopUpRow::Inter(states) = self.safe_row() {
            states.push(EntryState::new(EntryConfig::input(header)))
        }
        self
    }

    pub fn horizontal(
        &mut self,
        header: impl Into<String>,
        options: Option<impl IntoIterator<Item = impl Into<String>>>,
    ) -> &mut Self {
        if let PopUpRow::Inter(states) = self.safe_row() {
            match options {
                Some(to_options) => states.push(
                    EntryState::new(EntryConfig::horizontal(header)).with_options(to_options),
                ),
                None => states.push(EntryState::new(EntryConfig::horizontal(header))),
            }
        }
        self
    }

    pub fn vertical(
        &mut self,
        header: impl Into<String>,
        options: Option<impl IntoIterator<Item = impl Into<String>>>,
    ) -> &mut Self {
        if let PopUpRow::Inter(states) = self.safe_row() {
            match options {
                Some(to_options) => states
                    .push(EntryState::new(EntryConfig::vertical(header)).with_options(to_options)),
                None => states.push(EntryState::new(EntryConfig::vertical(header))),
            }
        }
        self
    }

    fn safe_row(&mut self) -> &mut PopUpRow {
        if self.rows.is_empty() || !matches!(self.rows.last(), Some(PopUpRow::Inter(_))) {
            self.row();
        }
        match self.rows.last_mut() {
            Some(row) => row,
            None => unreachable!("Rows were just ensured to be non-empty"),
        }
    }
}

impl Actionable for PopUpState {
    fn next(&mut self) {
        if self.is_empty() {
            return;
        }
        let i = self.selected().map_or(0, Self::next_capped(self.len()));
        self.select(Some(i));
    }

    fn prev(&mut self) {
        if self.is_empty() {
            return;
        }
        let i = self.selected().map_or(self.len() - 1, Self::prev_capped());
        self.select(Some(i));
    }

    fn select(&mut self, index: Option<usize>) {
        let ori = self.selected();
        if let Some(i) = self.selected()
            && let Some(row) = self.rows.get_mut(i)
        {
            row.unfocus();
        }
        self.focused = index;
        if let Some(i) = self.selected()
            && let Some(row) = self.rows.get_mut(i)
        {
            if ori < index {
                row.focus();
            } else {
                row.focus_last();
            }
        }
    }

    fn selected(&self) -> Option<usize> {
        self.focused
    }

    fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

impl PanelState for PopUpState {
    fn handle_key_events(&mut self, event: KeyEvent, context: AppContext) {
        match event.code {
            KeyCode::Tab | KeyCode::Enter => {
                let absorbed = self
                    .selected()
                    .and_then(|i| self.rows.get_mut(i))
                    .map(|r| r.next_inner())
                    .unwrap_or(false);

                if !absorbed {
                    self.next();
                }
            }
            KeyCode::BackTab => {
                let absorbed = self
                    .selected()
                    .and_then(|i| self.rows.get_mut(i))
                    .map(|r| r.prev_inner())
                    .unwrap_or(false);

                if !absorbed {
                    self.prev();
                }
            }
            _ => self.pass(event, context),
        }
    }

    fn pass(&mut self, event: KeyEvent, context: AppContext) {
        if let Some(i) = self.selected()
            && let Some(row) = self.rows.get_mut(i)
        {
            row.handle_key_events(event, context);
        }
    }
}

impl Hintable for PopUpState {
    fn hint(&mut self) -> &[(&str, &str)] {
        if let Some(i) = self.selected()
            && let Some(row) = self.rows.get_mut(i)
        {
            return row.hint();
        }
        Self::empty()
    }
}

// endregion
