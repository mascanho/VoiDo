use crate::arguments::models::Todo;
use crate::modals::{
    centered_rect, draw_delete_confirmation, draw_main_menu_modal, draw_priority_modal,
    draw_todo_modal,
};
use crate::search::InputField;
use crate::{App, database};
use ratatui::layout::Alignment;
use ratatui::prelude::Stylize;
use ratatui::text::Span;
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
    let area = f.size();

    // Color palette
    let background = Color::Rgb(25, 15, 30);
    let accent = Color::Rgb(150, 80, 220);
    let border = Color::Rgb(180, 140, 220);
    let text_primary = Color::Rgb(230, 220, 240);
    let text_secondary = Color::Rgb(200, 180, 220);
    let highlight = Color::Rgb(50, 30, 60);

    // Handle modal states first
    if app.show_delete_confirmation {
        draw_delete_confirmation(f, area);
        return;
    }
    if app.show_main_menu_modal {
        draw_main_menu_modal(f, area);
        return;
    }
    if app.show_priority_modal {
        draw_priority_modal(f, area);
        return;
    }
    if app.show_modal {
        draw_todo_modal(
            f,
            area,
            app.selected_todo.as_ref().unwrap(),
            &mut app.subtask_state,
        );
        return;
    }

    // Main layout with fixed search bar
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(1),    // Table
            Constraint::Length(2), // Stats
            Constraint::Length(1), // Shortcuts
        ])
        .split(area);

    // Create search block once and reuse reference
    let search_block = Block::default()
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(background));

    // Render search area (pass reference)
    f.render_widget(&search_block, layout[0]);
    app.search_input.render(f, search_block.inner(layout[0]));

    // Prepare table rows
    let rows = app.todos.iter().map(|todo| {
        Row::new(vec![
            todo.id.to_string().fg(text_primary),
            match todo.priority.to_lowercase().as_str() {
                "high" => todo.priority.clone().fg(Color::Rgb(220, 80, 150)),
                "medium" => todo.priority.clone().fg(Color::Rgb(180, 120, 120)),
                "low" => todo.priority.clone().fg(Color::Rgb(120, 220, 150)),
                _ => todo.priority.clone().fg(Color::Rgb(120, 80, 200)),
            },
            todo.topic.clone().fg(text_primary),
            todo.text.clone().fg(text_secondary),
            todo.subtasks.len().to_string().fg(text_secondary),
            todo.date_added.clone().fg(text_secondary),
            todo.due.clone().fg(text_secondary),
            match todo.status.as_str() {
                "Done" | "Completed" => todo.status.clone().fg(Color::Rgb(120, 220, 150)),
                "Ongoing" => todo.status.clone().fg(Color::Rgb(220, 180, 100)),
                "Planned" => todo.status.clone().fg(accent),
                "Pending" => todo.status.clone().fg(Color::Rgb(220, 100, 120)),
                _ => todo.status.clone().fg(text_primary),
            },
            todo.owner
                .clone()
                .fg(text_primary)
                .add_modifier(Modifier::ITALIC),
        ])
    });

    // Create and render table
    let table = Table::new(
        rows,
        [
            Constraint::Length(5),  // ID
            Constraint::Min(12),    // PRIORITY
            Constraint::Min(15),    // TOPIC
            Constraint::Fill(35),   // TODO
            Constraint::Length(8),  // SUBs
            Constraint::Length(12), // CREATED
            Constraint::Length(15), // DUE
            Constraint::Min(10),    // STATUS
            Constraint::Min(10),    // OWNER
        ],
    )
    .header(
        Row::new(vec![
            "ID", "PRIORITY", "TOPIC", "TODO", "SUBs", "CREATED", "DUE DATE", "STATUS", "OWNER",
        ])
        .style(Style::default().fg(accent).add_modifier(Modifier::BOLD)),
    )
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
            .fg(Color::White),
    )
    .column_spacing(1);

    f.render_stateful_widget(table, layout[1], &mut app.state);

    // Stats area
    let stats = calculate_stats(&app.todos);
    let stats_widget = Paragraph::new(stats).block(
        Block::default()
            .border_style(Style::default().fg(border))
            .style(Style::default().bg(background)),
    );
    f.render_widget(stats_widget, layout[2]);

    // Shortcuts area
    let shortcuts = get_shortcuts_text();
    let shortcuts_widget = Paragraph::new(shortcuts)
        .style(Style::default().fg(text_secondary))
        .block(Block::default().style(Style::default().bg(background)));
    f.render_widget(shortcuts_widget, layout[3]);
}

pub fn calculate_stats(todos: &[Todo]) -> Line {
    let done = todos.iter().filter(|t| t.status == "Done").count();
    let ongoing = todos.iter().filter(|t| t.status == "Ongoing").count();
    let pending = todos.iter().filter(|t| t.status == "Pending").count();

    Line::from(vec![
        Span::raw(" TOTAL: "),
        Span::styled(
            todos.len().to_string(),
            Style::default().fg(Color::Rgb(150, 80, 220)),
        ),
        Span::raw(" | Done: "),
        Span::styled(
            done.to_string(),
            Style::default().fg(Color::Rgb(120, 220, 150)),
        ),
        Span::raw(" | ONGOING: "),
        Span::styled(
            ongoing.to_string(),
            Style::default().fg(Color::Rgb(220, 180, 100)),
        ),
        Span::raw(" | PENDING: "),
        Span::styled(
            pending.to_string(),
            Style::default().fg(Color::Rgb(220, 100, 120)),
        ),
    ])
}

fn get_shortcuts_text() -> Line<'static> {
    Line::from(vec![
        Span::raw(" [↑/↓: Navigate] "),
        Span::raw(" [Enter: Details] "),
        Span::raw(" [M: Menu] "),
        Span::raw(" [q: Quit] "),
    ])
}
