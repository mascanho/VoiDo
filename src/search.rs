use crate::arguments::models::Todo;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::*,
    prelude::*,
    style::*,
    text::*,
    widgets::*,
};

use std::fmt;

impl fmt::Debug for FuzzySearch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FuzzySearch")
            .field("input", &self.input)
            .field("matched_indices", &self.matched_indices)
            .field("selected_match", &self.selected_match)
            .finish()
    }
}

pub struct FuzzySearch {
    matcher: SkimMatcherV2,
    pub input: InputField,
    matched_indices: Vec<usize>,
    selected_match: usize,
}

impl FuzzySearch {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
            input: InputField::new("Search"),
            matched_indices: Vec::new(),
            selected_match: 0,
        }
    }

    pub fn matched_indices(&self) -> &[usize] {
        &self.matched_indices
    }

    pub fn selected_match(&self) -> usize {
        self.selected_match
    }

    pub fn update_matches(&mut self, todos: &[Todo]) {
        self.matched_indices.clear();

        let search_text = &self.input.value;
        if search_text.is_empty() {
            // Show all items when search is empty
            self.matched_indices.extend(0..todos.len());
        } else {
            // TODO: Implement fuzzy matching with SkimMatcherV2
            // Fuzzy match against all todo fields
            for (idx, todo) in todos.iter().enumerate() {
                let combined_text = format!(
                    "{} {} {} {} {} {} {} {} {:?}",
                    todo.id,
                    todo.priority,
                    todo.topic,
                    todo.text,
                    todo.status,
                    todo.owner,
                    todo.notes,
                    todo.due,
                    todo.subtasks
                );
                if self
                    .matcher
                    .fuzzy_match(&combined_text, search_text)
                    .is_some()
                {
                    self.matched_indices.push(idx);
                }
            }
        }

        // Reset selection
        self.selected_match = if self.matched_indices.is_empty() {
            0
        } else {
            self.selected_match
                .min(self.matched_indices.len().saturating_sub(1))
        };
    }

    pub fn handle_event(&mut self, event: &Event) -> bool {
        if !self.input.active {
            return false;
        }

        // Handle input changes (typing, backspace, delete, ESC)
        let input_handled = if let Event::Key(_) = event {
            self.input.handle_event(event) // Pass all key events to InputField
        } else {
            false
        };

        // Handle navigation
        let navigation_handled = if let Event::Key(key) = event {
            match key.code {
                KeyCode::Down => {
                    if !self.matched_indices.is_empty() {
                        self.selected_match =
                            (self.selected_match + 1).min(self.matched_indices.len() - 1);
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Up => {
                    if !self.matched_indices.is_empty() {
                        self.selected_match = self.selected_match.saturating_sub(1);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        } else {
            false
        };

        input_handled || navigation_handled
    }
}

#[derive(Debug)]
pub struct InputField {
    pub value: String,
    pub cursor_position: usize,
    pub active: bool,
    pub title: String,
    pub background: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub multiline: bool,
    pub cursor_line: usize,
    pub cursor_col: usize,
}

impl InputField {
    pub fn new(title: &str) -> Self {
        Self {
            value: String::new(),
            cursor_position: 0,
            active: false, // Start inactive
            title: title.to_string(),
            background: Color::Rgb(30, 15, 35),
            border_color: Color::Rgb(180, 140, 220),
            text_color: Color::White,
            multiline: false,
            cursor_line: 0,
            cursor_col: 0,
        }
    }

    pub fn new_multiline(title: &str) -> Self {
        Self {
            value: String::new(),
            cursor_position: 0,
            active: false,
            title: title.to_string(),
            background: Color::Rgb(30, 15, 35),
            border_color: Color::Rgb(180, 140, 220),
            text_color: Color::White,
            multiline: true,
            cursor_line: 0,
            cursor_col: 0,
        }
    }

    pub fn focus(&mut self) {
        self.active = true;
        self.cursor_position = self.value.len(); // Move cursor to end
        if self.multiline {
            self.update_cursor_position();
        }
    }

    pub fn unfocus(&mut self) {
        self.active = false;
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // Add Borders::ALL to make the input field visible and interactive
        let input_block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL) // This was missing
            .style(Style::default().bg(self.background))
            .border_style(Style::default().fg(if self.active {
                self.border_color
            } else {
                Color::DarkGray
            }));

        let inner_area = input_block.inner(area);
        f.render_widget(input_block, area);

        if self.multiline {
            let text = Paragraph::new(self.value.as_str())
                .style(Style::default().fg(self.text_color))
                .wrap(ratatui::widgets::Wrap { trim: false });
            f.render_widget(text, inner_area);

            if self.active {
                let cursor_x = inner_area.x + self.cursor_col as u16;
                let cursor_y = inner_area.y + self.cursor_line as u16;
                if cursor_x < inner_area.x + inner_area.width
                    && cursor_y < inner_area.y + inner_area.height
                {
                    f.set_cursor(cursor_x, cursor_y);
                }
            }
        } else {
            let text = Paragraph::new(self.value.as_str())
                .style(Style::default().fg(self.text_color))
                .scroll((
                    0,
                    self.cursor_position
                        .saturating_sub(inner_area.width as usize) as u16,
                ));

            f.render_widget(text, inner_area);

            if self.active {
                let cursor_x = inner_area.x + (self.cursor_position as u16).min(inner_area.width);
                let cursor_y = inner_area.y;
                f.set_cursor(cursor_x, cursor_y);
            }
        }
    }

    pub fn handle_event(&mut self, event: &Event) -> bool {
        if !self.active {
            return false;
        }

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char(c) => {
                    self.value.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                    if self.multiline {
                        self.update_cursor_position();
                    }
                    return true;
                }
                KeyCode::Backspace => {
                    if self.cursor_position > 0 {
                        self.cursor_position -= 1;
                        self.value.remove(self.cursor_position);
                        if self.multiline {
                            self.update_cursor_position();
                        }
                        return true;
                    }
                }
                KeyCode::Left => {
                    if self.cursor_position > 0 {
                        self.cursor_position -= 1;
                        if self.multiline {
                            self.update_cursor_position();
                        }
                        return true;
                    }
                }
                KeyCode::Right => {
                    if self.cursor_position < self.value.len() {
                        self.cursor_position += 1;
                        if self.multiline {
                            self.update_cursor_position();
                        }
                        return true;
                    }
                }
                KeyCode::Delete => {
                    if self.cursor_position < self.value.len() {
                        self.value.remove(self.cursor_position);
                        if self.multiline {
                            self.update_cursor_position();
                        }
                        return true;
                    }
                }
                KeyCode::Home => {
                    if self.multiline {
                        // Move to beginning of current line
                        let lines: Vec<&str> = self.value.split('\n').collect();
                        let mut pos = 0;
                        for i in 0..self.cursor_line {
                            pos += lines[i].len() + 1;
                        }
                        self.cursor_position = pos;
                        self.cursor_col = 0;
                    } else {
                        self.cursor_position = 0;
                    }
                    return true;
                }
                KeyCode::End => {
                    if self.multiline {
                        // Move to end of current line
                        let lines: Vec<&str> = self.value.split('\n').collect();
                        let mut pos = 0;
                        for i in 0..self.cursor_line {
                            pos += lines[i].len() + 1;
                        }
                        pos += lines[self.cursor_line].len();
                        self.cursor_position = pos;
                        self.cursor_col = lines[self.cursor_line].len();
                    } else {
                        self.cursor_position = self.value.len();
                    }
                    return true;
                }
                KeyCode::Enter => {
                    if self.multiline {
                        self.value.insert(self.cursor_position, '\n');
                        self.cursor_position += 1;
                        self.update_cursor_position();
                        return true;
                    } else {
                        // Handle enter key if needed for single line
                        return true;
                    }
                }
                KeyCode::Up => {
                    if self.multiline && self.cursor_line > 0 {
                        self.move_cursor_up();
                        return true;
                    }
                }
                KeyCode::Down => {
                    if self.multiline {
                        self.move_cursor_down();
                        return true;
                    }
                }
                KeyCode::Esc => {
                    self.unfocus();
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
    }

    fn update_cursor_position(&mut self) {
        if !self.multiline {
            return;
        }

        let lines: Vec<&str> = self.value.split('\n').collect();
        if lines.is_empty() {
            self.cursor_line = 0;
            self.cursor_col = 0;
            return;
        }

        let mut pos = 0;
        self.cursor_line = 0;
        self.cursor_col = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            if pos + line.len() >= self.cursor_position {
                self.cursor_line = line_idx;
                self.cursor_col = self.cursor_position - pos;
                return;
            }
            pos += line.len() + 1; // +1 for the newline character
        }

        // If we reach here, cursor is at the very end
        if !lines.is_empty() {
            self.cursor_line = lines.len() - 1;
            self.cursor_col = lines[self.cursor_line].len();
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_line == 0 {
            return;
        }

        let lines: Vec<&str> = self.value.split('\n').collect();
        self.cursor_line -= 1;
        let target_line = lines[self.cursor_line];
        self.cursor_col = self.cursor_col.min(target_line.len());

        // Calculate new cursor position
        let mut pos = 0;
        for i in 0..self.cursor_line {
            pos += lines[i].len() + 1; // +1 for newline
        }
        pos += self.cursor_col;
        self.cursor_position = pos;
    }

    fn move_cursor_down(&mut self) {
        let lines: Vec<&str> = self.value.split('\n').collect();
        if self.cursor_line >= lines.len() - 1 {
            return;
        }

        self.cursor_line += 1;
        let target_line = lines[self.cursor_line];
        self.cursor_col = self.cursor_col.min(target_line.len());

        // Calculate new cursor position
        let mut pos = 0;
        for i in 0..self.cursor_line {
            pos += lines[i].len() + 1; // +1 for newline
        }
        pos += self.cursor_col;
        self.cursor_position = pos;
    }
}
