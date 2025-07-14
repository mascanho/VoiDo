use ratatui::layout::Alignment;
use ratatui::prelude::Stylize;
use ratatui::style::Styled;
use ratatui::text::Span;
use ratatui::widgets::{Padding, TableState};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
};

use crate::arguments::models::Todo;

// Dynamic sizing helper function
fn dynamic_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let width = (area.width * width_percent / 100).max(10); // Ensure minimum width
    let height = (area.height * height_percent / 100).max(8); // Ensure minimum height

    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;

    Rect::new(x, y, width, height)
}

pub fn draw_todo_modal(f: &mut Frame, area: Rect, todo: &Todo) {
    // Elegant purple color palette
    let background = Color::Rgb(25, 15, 30); // Deep purple
    let accent = Color::Rgb(150, 80, 220); // Vibrant purple
    let border = Color::Rgb(180, 140, 220); // Soft lavender
    let text_primary = Color::Rgb(230, 220, 240); // Light lavender
    let text_secondary = Color::Rgb(200, 180, 220); // Muted lavender

    // Main modal block with elegant styling
    let block = Block::default()
        .title(" TODO DETAILS ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(background).fg(text_primary));

    let area = centered_rect(100, 100, area);
    f.render_widget(block, area);

    let inner_area = area.inner(Margin {
        vertical: 2,
        horizontal: 3,
    });

    // Create styled text with purple color scheme
    let text = vec![
        Line::from(vec![
            "ID: ".fg(text_secondary),
            todo.id.to_string().bold().fg(accent),
        ]),
        Line::from(vec![
            "PRIORITY: ".fg(text_secondary),
            match todo.priority.to_lowercase().as_str() {
                "high" => todo.priority.as_str().bold().fg(Color::Rgb(220, 80, 150)), // Pinkish purple
                "medium" => todo.priority.as_str().bold().fg(Color::Rgb(180, 120, 120)), // Medium purple
                _ => todo.priority.as_str().bold().fg(Color::Rgb(120, 80, 200)), // Deep purple
            },
        ]),
        Line::from(vec![
            "Owner: ".fg(text_secondary),
            todo.owner.as_str().bold().fg(accent),
        ]),
        Line::from(vec![
            "TOPIC: ".fg(text_secondary),
            todo.topic.as_str().bold().fg(accent),
        ]),
        Line::from(vec![
            "STATUS: ".fg(text_secondary),
            match todo.status.as_str() {
                "Done" | "Completed" => todo.status.as_str().bold().fg(Color::Rgb(120, 220, 150)), // Soft green
                "Ongoing" => todo.status.as_str().bold().fg(Color::Rgb(220, 180, 100)), // Amber
                "Planned" => todo.status.as_str().bold().fg(accent),
                "Pending" => todo.status.as_str().bold().fg(Color::Rgb(220, 100, 120)), // Soft red
                _ => todo.status.as_str().bold().fg(accent),
            },
        ]),
        Line::from(vec![
            "CREATED: ".fg(text_secondary),
            todo.date_added.as_str().bold().fg(text_primary),
        ]),
        Line::from(vec![
            "DUE: ".fg(text_secondary),
            todo.due.as_str().bold().fg(text_primary),
        ]),
        Line::from(vec![
            "TODO: ".fg(text_secondary),
            todo.text.as_str().bold().fg(text_primary),
        ]),
        Line::from(vec![
            "DESCRIPTION: ".fg(text_secondary),
            todo.desc.as_str().bold().fg(text_primary),
        ]),
    ];

    // Paragraph with subtle styling
    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(Block::default().style(Style::default().bg(background)));

    // Split the inner area to make space for the subtasks
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(inner_area);

    f.render_widget(paragraph, layout[0]);

    // Create a list for subtasks
    let subtask_list: Vec<Line> = todo
        .subtasks
        .iter()
        .enumerate()
        .map(|(index, subtask)| {
            Line::from(vec![
                Span::styled(
                    format!("{}. ", index + 1),
                    Style::default().fg(Color::Rgb(180, 140, 220)),
                ),
                if subtask.status == "Done" || subtask.status == "Completed" {
                    Span::styled(
                        subtask.text.as_str(),
                        Style::default().fg(Color::Rgb(120, 220, 150)),
                    )
                    .crossed_out()
                } else {
                    Span::styled(subtask.text.as_str(), Style::default().fg(Color::Red))
                },
            ])
        })
        .collect();

    let title = format!(" Subtasks #{} ", todo.subtasks.len());
    // add margin to the subtasks paragraph
    let subtasks_paragraph = Paragraph::new(subtask_list)
        .block(
            Block::default()
                .title(title)
                .fg(Color::Rgb(180, 140, 220))
                .borders(Borders::ALL)
                .padding(Padding::new(1, 1, 0, 1))
                .style(Style::default().bg(background).fg(text_primary)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(subtasks_paragraph, layout[1]);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// DELETE CONFIRMATION MODAL
pub fn draw_delete_confirmation(f: &mut Frame, area: Rect) {
    // Purple-themed delete confirmation
    let background = Color::Rgb(30, 15, 35); // Slightly darker purple
    let border = Color::Rgb(200, 100, 220); // Bright purple border for warning
    let text_primary = Color::Rgb(230, 220, 240); // Light lavender
    let text_secondary = Color::Rgb(200, 180, 220); // Muted lavender

    let block = Block::default()
        .title(" Confirm Delete ")
        .borders(Borders::ALL)
        .style(Style::default().bg(background))
        .border_style(Style::default().fg(border).add_modifier(Modifier::BOLD));

    let area = centered_rect(40, 20, area);
    f.render_widget(block, area);

    let text = vec![
        Line::from("Are you sure you want to delete this item?".fg(text_primary)),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Y",
                Style::default()
                    .fg(Color::Rgb(120, 220, 150)) // Soft green
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(": Yes, delete".fg(text_secondary)),
        ]),
        Line::from(vec![
            Span::styled(
                "N",
                Style::default()
                    .fg(Color::Rgb(220, 100, 120)) // Soft red
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(": Cancel".fg(text_secondary)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(Block::default().style(Style::default().bg(background)));

    f.render_widget(paragraph, area);
}

// Status change confirmation
pub fn draw_priority_modal(f: &mut Frame, area: Rect) {
    // Purple-themed delete confirmation
    let background = Color::Rgb(30, 15, 35);
    let border = Color::Rgb(200, 100, 220);
    let text_primary = Color::Rgb(230, 220, 240);
    let text_secondary = Color::Rgb(200, 180, 220);

    // Calculate dynamic size (40% of width, 25% of height)
    let modal_area = dynamic_rect(40, 25, area);

    let block = Block::default()
        .title(" Priority Change ")
        .borders(Borders::ALL)
        .style(Style::default().bg(background))
        .border_style(Style::default().fg(border).add_modifier(Modifier::BOLD));

    f.render_widget(block, modal_area);

    // Inner area with padding
    let inner_area = modal_area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    let text = vec![
        Line::from(""),
        Line::from("Set priority for this TODO".fg(text_primary)),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "H",
                Style::default()
                    .fg(Color::Rgb(220, 100, 120))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(": High priority".fg(text_secondary)),
        ]),
        Line::from(vec![
            Span::styled(
                "M",
                Style::default()
                    .fg(Color::Rgb(220, 180, 100))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(": Medium priority".fg(text_secondary)),
        ]),
        Line::from(vec![
            Span::styled(
                "L",
                Style::default()
                    .fg(Color::Rgb(120, 220, 150))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(": Low priority".fg(text_secondary)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(Block::default().style(Style::default().bg(background)));

    f.render_widget(paragraph, inner_area);
}

// MAIN MODAL MENU
pub fn draw_main_menu_modal(f: &mut Frame, area: Rect) {
    // Purple-themed delete confirmation
    let background = Color::Rgb(30, 15, 35);
    let border = Color::Rgb(200, 100, 220);
    let text_primary = Color::Rgb(230, 220, 240);
    let text_secondary = Color::Rgb(200, 180, 220);

    // Calculate dynamic size (40% of width, 25% of height)
    let modal_area = dynamic_rect(40, 25, area);

    let block = Block::default()
        .title(" Main Menu ")
        .borders(Borders::ALL)
        .style(Style::default().bg(background))
        .border_style(Style::default().fg(border).add_modifier(Modifier::BOLD));

    f.render_widget(block, modal_area);

    // Inner area with padding
    let inner_area = modal_area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    let text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "E",
                Style::default()
                    .fg(Color::Rgb(220, 100, 120))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(": Export Todos to .xls".fg(text_secondary)),
        ]),
        Line::from(vec![
            Span::styled(
                "M",
                Style::default()
                    .fg(Color::Rgb(220, 180, 100))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(": Medium priority".fg(text_secondary)),
        ]),
        Line::from(vec![
            Span::styled(
                "L",
                Style::default()
                    .fg(Color::Rgb(120, 220, 150))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(": Low priority".fg(text_secondary)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(Block::default().style(Style::default().bg(background)));

    f.render_widget(paragraph, inner_area);
}
