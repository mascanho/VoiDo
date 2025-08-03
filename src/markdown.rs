use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub struct MarkdownRenderer {
    pub accent_color: Color,
    pub text_color: Color,
    pub secondary_color: Color,
    pub bold_color: Color,
    pub italic_color: Color,
    pub code_color: Color,
    pub heading_color: Color,
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self {
            accent_color: Color::Rgb(150, 80, 220),
            text_color: Color::Rgb(230, 220, 240),
            secondary_color: Color::Rgb(200, 180, 220),
            bold_color: Color::Rgb(255, 255, 255),
            italic_color: Color::Rgb(180, 140, 220),
            code_color: Color::Rgb(120, 220, 150),
            heading_color: Color::Rgb(220, 180, 100),
        }
    }
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&self, markdown: &str) -> Vec<Line> {
        if markdown.is_empty() {
            return vec![Line::from("")];
        }

        let parser = Parser::new(markdown);
        let mut lines = Vec::new();
        let mut current_line = Vec::new();
        let mut style_stack = Vec::new();
        let mut in_code_block = false;
        let mut code_block_lang = String::new();

        for event in parser {
            match event {
                Event::Start(tag) => {
                    let style = self.get_style_for_tag(&tag);
                    style_stack.push(style);

                    match tag {
                        Tag::Heading { level, .. } => {
                            if !current_line.is_empty() {
                                lines.push(Line::from(current_line.clone()));
                                current_line.clear();
                            }
                            let prefix = "#".repeat(level as usize);
                            current_line.push(Span::styled(
                                format!("{} ", prefix),
                                Style::default()
                                    .fg(self.heading_color)
                                    .add_modifier(Modifier::BOLD),
                            ));
                        }
                        Tag::CodeBlock(kind) => {
                            in_code_block = true;
                            if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                                code_block_lang = lang.to_string();
                            }
                            if !current_line.is_empty() {
                                lines.push(Line::from(current_line.clone()));
                                current_line.clear();
                            }
                            if !code_block_lang.is_empty() {
                                current_line.push(Span::styled(
                                    format!("```{}", code_block_lang),
                                    Style::default().fg(self.secondary_color),
                                ));
                                lines.push(Line::from(current_line.clone()));
                                current_line.clear();
                            }
                        }
                        Tag::Paragraph => {
                            if !current_line.is_empty() {
                                lines.push(Line::from(current_line.clone()));
                                current_line.clear();
                            }
                        }
                        Tag::List(_) => {
                            if !current_line.is_empty() {
                                lines.push(Line::from(current_line.clone()));
                                current_line.clear();
                            }
                        }
                        Tag::Item => {
                            current_line
                                .push(Span::styled("• ", Style::default().fg(self.accent_color)));
                        }
                        Tag::BlockQuote(_) => {
                            current_line.push(Span::styled(
                                "│ ",
                                Style::default().fg(self.secondary_color),
                            ));
                        }
                        _ => {}
                    }
                }
                Event::End(tag_end) => {
                    style_stack.pop();

                    match tag_end {
                        TagEnd::Heading(_) => {
                            lines.push(Line::from(current_line.clone()));
                            current_line.clear();
                            lines.push(Line::from(""));
                        }
                        TagEnd::CodeBlock => {
                            in_code_block = false;
                            if !code_block_lang.is_empty() {
                                current_line.push(Span::styled(
                                    "```",
                                    Style::default().fg(self.secondary_color),
                                ));
                                code_block_lang.clear();
                            }
                            lines.push(Line::from(current_line.clone()));
                            current_line.clear();
                            lines.push(Line::from(""));
                        }
                        TagEnd::Paragraph => {
                            lines.push(Line::from(current_line.clone()));
                            current_line.clear();
                            lines.push(Line::from(""));
                        }
                        TagEnd::List(_) => {
                            if !current_line.is_empty() {
                                lines.push(Line::from(current_line.clone()));
                                current_line.clear();
                            }
                            lines.push(Line::from(""));
                        }
                        TagEnd::Item => {
                            lines.push(Line::from(current_line.clone()));
                            current_line.clear();
                        }
                        TagEnd::BlockQuote(_) => {
                            lines.push(Line::from(current_line.clone()));
                            current_line.clear();
                            lines.push(Line::from(""));
                        }
                        _ => {}
                    }
                }
                Event::Text(text) => {
                    let current_style = style_stack
                        .last()
                        .copied()
                        .unwrap_or_else(|| Style::default().fg(self.text_color));

                    if in_code_block {
                        // In code blocks, preserve formatting and use monospace styling
                        for line in text.lines() {
                            if !current_line.is_empty() || !line.is_empty() {
                                current_line.push(Span::styled(
                                    line.to_string(),
                                    Style::default().fg(self.code_color),
                                ));
                            }
                            if text.contains('\n') && line != text.lines().last().unwrap() {
                                lines.push(Line::from(current_line.clone()));
                                current_line.clear();
                            }
                        }
                    } else {
                        // Regular text - handle line breaks
                        for (i, line) in text.lines().enumerate() {
                            if i > 0 {
                                lines.push(Line::from(current_line.clone()));
                                current_line.clear();
                            }
                            if !line.is_empty() {
                                current_line.push(Span::styled(line.to_string(), current_style));
                            }
                        }
                    }
                }
                Event::Code(code) => {
                    let style = Style::default()
                        .fg(self.code_color)
                        .bg(Color::Rgb(40, 40, 60));
                    current_line.push(Span::styled(format!("`{}`", code), style));
                }
                Event::Html(html) => {
                    // Basic HTML support - just render as text with different color
                    current_line.push(Span::styled(
                        html.to_string(),
                        Style::default().fg(self.secondary_color),
                    ));
                }
                Event::SoftBreak => {
                    current_line.push(Span::raw(" "));
                }
                Event::HardBreak => {
                    lines.push(Line::from(current_line.clone()));
                    current_line.clear();
                }
                Event::Rule => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                    }
                    lines.push(Line::from(Span::styled(
                        "─".repeat(50),
                        Style::default().fg(self.secondary_color),
                    )));
                    lines.push(Line::from(""));
                }
                _ => {}
            }
        }

        // Add any remaining content
        if !current_line.is_empty() {
            lines.push(Line::from(current_line));
        }

        // Remove trailing empty lines but keep at least one line
        while lines.len() > 1 && lines.last().map_or(false, |line| line.spans.is_empty()) {
            lines.pop();
        }

        if lines.is_empty() {
            lines.push(Line::from(""));
        }

        lines
    }

    fn get_style_for_tag(&self, tag: &Tag) -> Style {
        match tag {
            Tag::Emphasis => Style::default()
                .fg(self.italic_color)
                .add_modifier(Modifier::ITALIC),
            Tag::Strong => Style::default()
                .fg(self.bold_color)
                .add_modifier(Modifier::BOLD),
            Tag::Strikethrough => Style::default()
                .fg(self.secondary_color)
                .add_modifier(Modifier::CROSSED_OUT),
            Tag::Link { .. } => Style::default()
                .fg(self.accent_color)
                .add_modifier(Modifier::UNDERLINED),
            Tag::Heading { .. } => Style::default()
                .fg(self.heading_color)
                .add_modifier(Modifier::BOLD),
            Tag::BlockQuote(_) => Style::default()
                .fg(self.secondary_color)
                .add_modifier(Modifier::ITALIC),
            _ => Style::default().fg(self.text_color),
        }
    }

    // Helper method to render markdown for editing (plain text with syntax highlighting)
    pub fn render_for_editing(
        &self,
        markdown: &str,
        cursor_line: usize,
        cursor_col: usize,
    ) -> Vec<Line> {
        let lines: Vec<&str> = markdown.split('\n').collect();
        let mut result = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            if line_idx == cursor_line {
                // Add cursor to this line
                let mut line_with_cursor = line.to_string();
                let cursor_pos = cursor_col.min(line.len());
                line_with_cursor.insert(cursor_pos, '█');
                result.push(self.highlight_markdown_syntax(&line_with_cursor));
            } else {
                result.push(self.highlight_markdown_syntax(line));
            }
        }

        // If cursor is beyond all lines, add a new line with cursor
        if cursor_line >= lines.len() {
            result.push(Line::from(Span::styled(
                "█",
                Style::default().fg(self.text_color),
            )));
        }

        if result.is_empty() {
            result.push(Line::from(""));
        }

        result
    }

    fn highlight_markdown_syntax(&self, line: &str) -> Line {
        let mut spans = Vec::new();
        let mut chars = line.chars().peekable();
        let mut current_text = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                '#' if current_text.is_empty() => {
                    // Heading syntax
                    let mut level = 1;
                    while chars.peek() == Some(&'#') {
                        chars.next();
                        level += 1;
                    }
                    if chars.peek() == Some(&' ') {
                        chars.next(); // consume space
                        spans.push(Span::styled(
                            "#".repeat(level) + " ",
                            Style::default()
                                .fg(self.heading_color)
                                .add_modifier(Modifier::BOLD),
                        ));
                    } else {
                        current_text.push_str(&"#".repeat(level));
                    }
                }
                '*' | '_' => {
                    if !current_text.is_empty() {
                        spans.push(Span::styled(
                            current_text.clone(),
                            Style::default().fg(self.text_color),
                        ));
                        current_text.clear();
                    }
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default()
                            .fg(self.accent_color)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                '`' => {
                    if !current_text.is_empty() {
                        spans.push(Span::styled(
                            current_text.clone(),
                            Style::default().fg(self.text_color),
                        ));
                        current_text.clear();
                    }
                    spans.push(Span::styled(
                        "`",
                        Style::default()
                            .fg(self.code_color)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                '[' | ']' | '(' | ')' => {
                    if !current_text.is_empty() {
                        spans.push(Span::styled(
                            current_text.clone(),
                            Style::default().fg(self.text_color),
                        ));
                        current_text.clear();
                    }
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(self.accent_color),
                    ));
                }
                '>' if current_text.is_empty() => {
                    spans.push(Span::styled(
                        "> ",
                        Style::default()
                            .fg(self.secondary_color)
                            .add_modifier(Modifier::BOLD),
                    ));
                    if chars.peek() == Some(&' ') {
                        chars.next();
                    }
                }
                '-' if current_text.is_empty() && chars.peek() == Some(&' ') => {
                    chars.next(); // consume space
                    spans.push(Span::styled(
                        "- ",
                        Style::default()
                            .fg(self.accent_color)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                '█' => {
                    // Cursor character - keep original styling
                    if !current_text.is_empty() {
                        spans.push(Span::styled(
                            current_text.clone(),
                            Style::default().fg(self.text_color),
                        ));
                        current_text.clear();
                    }
                    spans.push(Span::styled("█", Style::default().fg(self.text_color)));
                }
                _ => {
                    current_text.push(ch);
                }
            }
        }

        if !current_text.is_empty() {
            spans.push(Span::styled(
                current_text,
                Style::default().fg(self.text_color),
            ));
        }

        if spans.is_empty() {
            spans.push(Span::raw(""));
        }

        Line::from(spans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown_rendering() {
        let renderer = MarkdownRenderer::new();
        let markdown = "# Heading\n\nThis is **bold** and *italic* text.";
        let lines = renderer.render(markdown);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_empty_markdown() {
        let renderer = MarkdownRenderer::new();
        let lines = renderer.render("");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_code_block() {
        let renderer = MarkdownRenderer::new();
        let markdown = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        let lines = renderer.render(markdown);
        assert!(!lines.is_empty());
    }
}
