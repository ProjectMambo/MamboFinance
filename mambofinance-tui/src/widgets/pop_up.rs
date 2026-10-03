use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use crate::{
    app::AppContext,
    widgets::{Actionable, Focusable, Gettable, PanelState, UIEvent, bottom_bar::Hintable},
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

    fn border_style(&self) -> Style {
        if self.is_focus() {
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

    pub fn compile(mut self) -> Option<String> {
        match self.config.kind {
            EntryKind::HorizontalOption | EntryKind::VerticalOption
                if let Some(idx) = self.selected() =>
            {
                Some(self.options.remove(idx))
            }
            EntryKind::Input => Some(self.value),
            _ => None,
        }
    }
}

impl Actionable for EntryState {
    fn next(&mut self) {
        if !self.is_empty() {
            self.select(self.next_capped());
        }
    }

    fn prev(&mut self) {
        if !self.is_empty() {
            self.select(self.prev_capped());
        }
    }

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
}

impl Focusable for EntryState {
    fn is_focus(&self) -> bool {
        self.focused
    }
    fn set_focus(&mut self, focus: bool) {
        self.focused = focus
    }
}

impl PanelState for EntryState {
    fn handle_key_events(
        &mut self,
        event: KeyEvent,
        #[allow(unused_variables)] context: &AppContext,
    ) -> bool {
        // consume every key to lock input to pop up
        match self.config.kind {
            EntryKind::HorizontalOption => match event.code {
                KeyCode::Char(c) => self.value.push(c),
                KeyCode::Backspace => {
                    self.value.clear();
                }
                KeyCode::Left => self.prev(),
                KeyCode::Right => self.next(),
                _ => return false,
            },
            EntryKind::VerticalOption => match event.code {
                KeyCode::Char(c) => self.value.push(c),
                KeyCode::Backspace => {
                    self.value.clear();
                }
                KeyCode::Up => self.prev(),
                KeyCode::Down => self.next(),
                _ => return false,
            },
            EntryKind::Input => match event.code {
                KeyCode::Char(c) => self.value.push(c),
                KeyCode::Backspace if event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.value.clear();
                }
                KeyCode::Backspace => {
                    self.value.pop();
                }
                _ => return false,
            },
        }
        true
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
            focused: Some(0),
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

    pub fn compile(self) -> Vec<Option<String>> {
        self.entries.into_iter().map(EntryState::compile).collect()
    }
}

impl Actionable for RowState {
    fn select(&mut self, index: Option<usize>) {
        self.unfocus();
        self.focused = index;
        self.focus();
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

impl Focusable for RowState {
    fn is_focus(&self) -> bool {
        if let Some(entry) = self.get() {
            entry.is_focus()
        } else {
            false
        }
    }
    fn set_focus(&mut self, focus: bool) {
        if let Some(entry) = self.get_mut() {
            entry.set_focus(focus);
        }
    }
    fn focus(&mut self) {
        self.unfocus();
        self.set_focus(true);
    }
    fn unfocus(&mut self) {
        self.entries.iter_mut().map(Focusable::unfocus).collect()
    }
}

impl Gettable<EntryState> for RowState {
    fn get(&self) -> Option<&EntryState> {
        self.selected().and_then(|i| self.entries.get(i))
    }
    fn get_mut(&mut self) -> Option<&mut EntryState> {
        self.selected().and_then(|i| self.entries.get_mut(i))
    }
}

impl PanelState for RowState {
    fn handle_key_events(&mut self, event: KeyEvent, context: &AppContext) -> bool {
        if !self.pass(event, context) {
            match event.code {
                KeyCode::Tab | KeyCode::Enter => {
                    self.next();
                }
                KeyCode::BackTab => {
                    self.prev();
                }
                _ => return false,
            }
        }
        true
    }

    fn pass(&mut self, event: KeyEvent, context: &AppContext) -> bool {
        if let Some(entry) = self.get_mut() {
            entry.handle_key_events(event, context)
        } else {
            false
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
pub struct PopUpBuilder {
    entries: Vec<EntryState>,
}

impl Default for PopUpBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PopUpBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn complete(self) -> RowState {
        RowState::new(self.entries)
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

#[derive(Debug, Clone)]
pub struct PopUpState {
    pub title: String,
    pub rows: Vec<RowState>,
    focused: Option<usize>,
    builder: Option<PopUpBuilder>,
}

impl PopUpState {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            rows: Vec::new(),
            focused: Some(0),
            builder: None,
        }
    }

    pub fn open(&mut self) {
        self.select(Some(0));
    }

    pub fn row(&mut self) -> &mut Self {
        self.complete();
        self.builder = Some(PopUpBuilder::new());
        self
    }

    pub fn complete(&mut self) -> &mut Self {
        if let Some(builder) = self.builder.take() {
            self.rows.push(builder.complete());
        }
        self
    }

    pub fn input(&mut self, header: impl Into<String>) -> &mut Self {
        if let Some(builder) = self.safe_row() {
            builder
                .entries
                .push(EntryState::new(EntryConfig::input(header)))
        }
        self
    }

    pub fn horizontal(
        &mut self,
        header: impl Into<String>,
        options: Option<impl IntoIterator<Item = impl Into<String>>>,
    ) -> &mut Self {
        if let Some(builder) = self.safe_row() {
            let mut entry = EntryState::new(EntryConfig::horizontal(header));
            if let Some(opts) = options {
                entry = entry.with_options(opts);
            }
            builder.entries.push(entry);
        }
        self
    }

    pub fn vertical(
        &mut self,
        header: impl Into<String>,
        options: Option<impl IntoIterator<Item = impl Into<String>>>,
    ) -> &mut Self {
        if let Some(builder) = self.safe_row() {
            let mut entry = EntryState::new(EntryConfig::vertical(header));
            if let Some(opts) = options {
                entry = entry.with_options(opts);
            }
            builder.entries.push(entry);
        }
        self
    }

    fn safe_row(&mut self) -> &mut Option<PopUpBuilder> {
        if self.is_empty() && self.builder.is_none() {
            self.row();
        }
        &mut self.builder
    }

    pub fn compile(self) -> Vec<Option<String>> {
        self.rows.into_iter().flat_map(RowState::compile).collect()
    }
}

impl Actionable for PopUpState {
    fn next(&mut self) {
        if !self.is_empty() {
            self.select(self.next_capped());
        }
    }

    fn prev(&mut self) {
        if !self.is_empty() {
            self.select(self.prev_capped());
        }
    }

    fn select(&mut self, index: Option<usize>) {
        self.unfocus();
        self.focused = index;
        self.focus();
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

    fn is_last(&self) -> bool {
        if let Some(entry) = self.get() {
            self.selected() == self.last() && entry.is_last()
        } else {
            false
        }
    }
}

impl Focusable for PopUpState {
    fn is_focus(&self) -> bool {
        if let Some(entry) = self.get() {
            entry.is_focus()
        } else {
            false
        }
    }
    fn set_focus(&mut self, focus: bool) {
        if let Some(entry) = self.get_mut() {
            entry.set_focus(focus);
        }
    }
    fn focus(&mut self) {
        self.unfocus();
        self.set_focus(true);
    }
    fn unfocus(&mut self) {
        self.rows.iter_mut().map(Focusable::unfocus).collect()
    }
}

impl Gettable<RowState> for PopUpState {
    fn get(&self) -> Option<&RowState> {
        self.selected().and_then(|i| self.rows.get(i))
    }
    fn get_mut(&mut self) -> Option<&mut RowState> {
        self.selected().and_then(|i| self.rows.get_mut(i))
    }
}

impl PanelState for PopUpState {
    fn handle_key_events(&mut self, event: KeyEvent, context: &AppContext) -> bool {
        if event.code == KeyCode::Esc {
            return false;
        }
        // handle pass inside for correct highlighting between different entries with next and prev
        // absorb everything except esc to lock input to pop up
        match event.code {
            KeyCode::Enter if self.is_last() => {
                let _ = context.event_sender.send(UIEvent::PopUp(PopUpEvent::Add));
            }
            KeyCode::Tab | KeyCode::Enter => {
                if let Some(row) = self.get_mut()
                    && row.is_last()
                {
                    self.next()
                } else {
                    self.pass(event, context);
                }
            }
            KeyCode::BackTab => {
                if let Some(row) = self.get_mut()
                    && row.is_first()
                {
                    self.prev()
                } else {
                    self.pass(event, context);
                }
            }
            _ => {
                self.pass(event, context);
            }
        }
        true
    }

    fn pass(&mut self, event: KeyEvent, context: &AppContext) -> bool {
        if let Some(row) = self.get_mut() {
            row.handle_key_events(event, context)
        } else {
            false
        }
    }
}

impl Hintable for PopUpState {
    fn hint(&mut self) -> &[(&str, &str)] {
        if let Some(row) = self.get_mut() {
            return row.hint();
        }
        Self::empty()
    }
}

// endregion

// region: Popable

pub trait Popable {
    fn is_pop(&self) -> bool;
    fn pop(&mut self, pop: bool);
}

// endregion

// region: PopUpEvent

pub enum PopUpEvent {
    Add,
    Edit,
    Delete,
}

// endregion
