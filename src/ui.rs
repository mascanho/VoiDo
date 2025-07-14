use crate::App;
use crate::arguments::models::Todo;
use crate::modals::{
    centered_rect, draw_delete_confirmation, draw_main_menu_modal, draw_priority_modal,
    draw_todo_modal,
};
use ratatui::layout::Alignment;
use ratatui::prelude::Stylize;
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

// MAIN UI
pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Elegant purple color palette
    let background = Color::Rgb(25, 15, 30); // Deep purple
    let accent = Color::Rgb(150, 80, 220); // Vibrant purple
    let border = Color::Rgb(180, 140, 220); // Soft lavender
    let text_primary = Color::Rgb(230, 220, 240); // Light lavender
    let text_secondary = Color::Rgb(200, 180, 220); // Muted lavender
    let highlight = Color::Rgb(50, 30, 60); // Darker purple

    // Handle modal and delete confirmation states first
    if app.show_delete_confirmation {
        draw_delete_confirmation(f, area);
        return;
    }

    // Show main menu modal
    if app.show_main_menu_modal {
        draw_main_menu_modal(f, area);
        return;
    }

    // Show priority modal
    if app.show_priority_modal {
        draw_priority_modal(f, area);
        return;
    }

    // show deletion modal for selected todo
    if app.show_modal {
        draw_todo_modal(f, area, app.selected_todo.as_ref().unwrap());
        return;
    }

    // Main table view layout
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Main table area
            Constraint::Length(3), // Stats area
            Constraint::Length(1), // Shortcuts area
        ])
        .split(area);

    // Elegant table header
    let header = Row::new(vec![
        "ID", "PRIORITY", "TOPIC", "TODO", "CREATED", "DUE DATE", "STATUS", "OWNER",
    ])
    .style(Style::default().fg(accent).add_modifier(Modifier::BOLD));

    // Table rows with status-based coloring
    let rows = app.todos.iter().map(|todo| {
        Row::new(vec![
            todo.id.to_string().fg(text_primary),
            match todo.priority.to_lowercase().as_str() {
                "high" | "High" | "HIGH" => todo.priority.clone().fg(Color::Rgb(220, 80, 150)), // Pinkish purple
                "medium" | "Medium" | "MEDIUM" => {
                    todo.priority.clone().fg(Color::Rgb(180, 120, 120))
                } // Medium Yellow
                "low" | "Low" | "LOW" => todo.priority.clone().fg(Color::Rgb(120, 80, 200)), // Deep purple
                _ => todo.priority.clone().fg(Color::Rgb(120, 80, 200)), // Deep purple
            },
            todo.topic.clone().fg(text_primary),
            todo.text.clone().fg(text_secondary),
            todo.date_added.clone().fg(text_secondary),
            todo.due.clone().fg(text_secondary),
            match todo.status.as_str() {
                "Done" | "Completed" => todo.status.clone().fg(Color::Rgb(120, 220, 150)), // Soft green
                "Ongoing" => todo.status.clone().fg(Color::Rgb(220, 180, 100)),            // Amber
                "Planned" => todo.status.clone().fg(accent),
                "Pending" => todo.status.clone().fg(Color::Rgb(220, 100, 120)), // Soft red
                _ => todo.status.clone().fg(text_primary),
            },
            todo.owner
                .clone()
                .fg(text_primary)
                .style(Style::default().add_modifier(Modifier::ITALIC)),
        ])
    });

    // Elegant table styling
    let table = Table::new(
        rows.collect::<Vec<_>>(),
        vec![
            Constraint::Length(5),  // ID
            Constraint::Min(12),    // PRIORITY
            Constraint::Min(15),    // TOPIC
            Constraint::Fill(35),   // TODO-Text
            Constraint::Length(12), // DATE-created
            Constraint::Length(15), // DUE
            Constraint::Min(10),    // STATUS
            Constraint::Min(10),    // Owner
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" VoiDo ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border))
            .style(Style::default().bg(background)),
    )
    .highlight_style(Style::default().bg(highlight).fg(text_primary))
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(120, 80, 190))
            .fg(Color::Rgb(255, 255, 255)),
    )
    .column_spacing(1);

    f.render_stateful_widget(table, layout[0], &mut app.state);

    // Stats with elegant styling
    let stats_spans = calculate_stats(&app.todos);
    let status_line = Paragraph::new(Line::from(stats_spans)).block(
        Block::default()
            .border_style(Style::default().fg(border))
            .style(Style::default().bg(background)),
    );

    f.render_widget(status_line, layout[1]);

    // Shortcuts with consistent styling
    let shortcuts = Paragraph::new(get_shortcuts_text())
        .style(Style::default().fg(text_secondary))
        .block(Block::default().style(Style::default().bg(background)));

    f.render_widget(shortcuts, layout[2]);
}

pub fn calculate_stats(todos: &[Todo]) -> Vec<Span<'static>> {
    let done = todos.iter().filter(|t| t.status == "Done").count();
    let ongoing = todos.iter().filter(|t| t.status == "Ongoing").count();
    // let planned = todos.iter().filter(|t| t.status == "Planned").count();
    let pending = todos.iter().filter(|t| t.status == "Pending").count();

    vec![
        Span::raw(" "),
        Span::raw("TOTAL: "),
        Span::styled(
            todos.len().to_string(),
            Style::default().fg(Color::Rgb(150, 80, 220)), // Purple
        ),
        Span::raw(" | Done: "),
        Span::styled(
            done.to_string(),
            Style::default().fg(Color::Rgb(120, 220, 150)), // Green
        ),
        Span::raw(" | ONGOING: "),
        Span::styled(
            ongoing.to_string(),
            Style::default().fg(Color::Rgb(220, 180, 100)), // Amber
        ),
        // Span::raw(" | PLANNED: "),
        // Span::styled(
        //     planned.to_string(),
        //     Style::default().fg(Color::Rgb(180, 140, 220)), // Lavender
        // ),
        Span::raw(" | PENDING: "),
        Span::styled(
            pending.to_string(),
            Style::default().fg(Color::Rgb(220, 100, 120)), // Red
        ),
    ]
}

// KEYWBOARD SHORTCUTS
fn get_shortcuts_text() -> Line<'static> {
    Line::from(vec![
        " ".into(),
        "[ ".into(),
        "↑/↓: Navigate".into(),
        " ]".into(),
        " ".into(),
        "[ ".into(),
        "Enter: Details".into(),
        " ]".into(),
        " ".into(),
        "[ ".into(),
        "delete: Delete".into(),
        " ]".into(),
        " ".into(),
        "[ ".into(),
        "d: Done".into(),
        " ] ".into(),
        "[ ".into(),
        "p: Pending".into(),
        " ] ".into(),
        "[ ".into(),
        "o: Ongoing".into(),
        " ] ".into(),
        "[ ".into(),
        "P: Priority".into(),
        " ] ".into(),
        "[ ".into(),
        "M: Menu".into(),
        " ] ".into(),
        "[ ".into(),
        "q: Quit".into(),
        " ]".into(),
    ])
}
