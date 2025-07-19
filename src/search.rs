use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::*,
    prelude::*,
    style::*,
    text::*,
    widgets::*,
};

#[derive(Debug)]
pub struct InputField {
    pub value: String,
    pub cursor_position: usize,
    pub active: bool,
    pub title: String,
    pub background: Color,
    pub border_color: Color,
    pub text_color: Color,
}

impl InputField {
    pub fn new(title: &str) -> Self {
        Self {
            value: String::new(),
            cursor_position: 0,
            active: true,
            title: title.to_string(),
            background: Color::Rgb(30, 15, 35),
            border_color: Color::Rgb(200, 100, 220),
            text_color: Color::White,
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // Add Borders::ALL to make the input field visible and interactive
        let input_block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL) // This was missing
            .style(Style::default().bg(self.background))
            .border_style(Style::default().fg(if self.active {
                self.border_color
            } else {
                Color::DarkGray
            }));

        let inner_area = input_block.inner(area);
        f.render_widget(input_block, area);

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

    pub fn handle_event(&mut self, event: &Event) -> bool {
        if !self.active {
            return false;
        }

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char(c) => {
                    self.value.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                    return true;
                }
                KeyCode::Backspace => {
                    if self.cursor_position > 0 {
                        self.cursor_position -= 1;
                        self.value.remove(self.cursor_position);
                        return true;
                    }
                }
                KeyCode::Left => {
                    if self.cursor_position > 0 {
                        self.cursor_position -= 1;
                        return true;
                    }
                }
                KeyCode::Right => {
                    if self.cursor_position < self.value.len() {
                        self.cursor_position += 1;
                        return true;
                    }
                }
                KeyCode::Delete => {
                    if self.cursor_position < self.value.len() {
                        self.value.remove(self.cursor_position);
                        return true;
                    }
                }
                KeyCode::Home => {
                    self.cursor_position = 0;
                    return true;
                }
                KeyCode::End => {
                    self.cursor_position = self.value.len();
                    return true;
                }
                KeyCode::Enter => {
                    // Handle enter key if needed
                    return true;
                }
                KeyCode::Esc => {
                    // Handle escape key if needed
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
}
