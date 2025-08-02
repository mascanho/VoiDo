use ratatui::layout::Alignment;
use ratatui::prelude::Stylize;
use ratatui::text::Span;
use ratatui::widgets::{List, ListItem, ListState, Padding};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
};

use crate::arguments::models::Todo;

// Dynamic sizing helper function
pub fn dynamic_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let width = (area.width * width_percent / 100).max(10); // Ensure minimum width
    let height = (area.height * height_percent / 100).max(8); // Ensure minimum height

    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;

    Rect::new(x, y, width, height)
}

pub fn draw_todo_modal(
    f: &mut Frame,
    area: Rect,
    todo: &Todo,
    state: &mut ListState,
    editing_notes: bool,
    notes_input: &crate::search::InputField,
    notes_scroll_offset: u16,
) {
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
        vertical: 3,
        horizontal: 4,
    });

    // Create styled text with purple color scheme and better spacing
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
                "low" => todo.priority.as_str().bold().fg(Color::Rgb(120, 220, 150)), // Soft green
                _ => todo.priority.as_str().bold().fg(Color::Rgb(120, 80, 200)),      // Deep purple
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

    // Split the inner area horizontally first with better proportions and spacing
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(60),
                Constraint::Min(2),
                Constraint::Percentage(38),
            ]
            .as_ref(),
        )
        .split(inner_area);

    // Split the left area vertically for main content and subtasks with more balanced spacing
    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Min(1),
                Constraint::Percentage(48),
            ]
            .as_ref(),
        )
        .split(horizontal_layout[0]);

    // Render main todo information in the top-left with padding
    let main_content_area = left_layout[0].inner(Margin {
        horizontal: 2,
        vertical: 1,
    });
    f.render_widget(paragraph, main_content_area);

    // Create notes section in the right panel with better spacing
    let notes_area = horizontal_layout[2].inner(Margin {
        horizontal: 1,
        vertical: 0,
    });

    if editing_notes {
        // Create a block for the notes editing area
        let notes_block = Block::default()
            .title(" Editing Notes ")
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Rgb(220, 180, 100))
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(background).fg(text_primary));

        // Render the block first
        f.render_widget(notes_block, notes_area);

        // Get the inner area for the input content
        let inner_notes_area = notes_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        // Create the header line
        let header =
            Paragraph::new("NOTES (ESC to save):").style(Style::default().fg(text_secondary));

        // Split the inner area for header and content
        let notes_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)].as_ref())
            .split(inner_notes_area);

        f.render_widget(header, notes_layout[0]);

        // Render the input content with cursor
        let content_area = notes_layout[1];
        let visible_height = content_area.height;

        // Split notes by single newlines to preserve all line breaks
        let lines: Vec<&str> = notes_input.value.split('\n').collect();
        let mut display_lines = Vec::new();

        // Calculate scroll bounds
        let total_lines = lines.len()
            + if notes_input.active && notes_input.cursor_line >= lines.len() {
                1
            } else {
                0
            };
        let max_scroll = if total_lines > visible_height as usize {
            total_lines - visible_height as usize
        } else {
            0
        };
        let scroll_offset = (notes_scroll_offset as usize).min(max_scroll);

        // Only render visible lines based on scroll offset
        let start_line = scroll_offset;
        let end_line = (start_line + visible_height as usize).min(lines.len());

        for line_idx in start_line..end_line {
            let line = lines[line_idx];
            if line_idx == notes_input.cursor_line && notes_input.active {
                // This is the line with the cursor - insert cursor character
                let mut line_with_cursor = line.to_string();
                let cursor_pos = notes_input.cursor_col.min(line.len());
                line_with_cursor.insert(cursor_pos, '█'); // Block cursor character
                display_lines.push(Line::from(line_with_cursor.fg(text_primary)));
            } else {
                display_lines.push(Line::from(line.fg(text_primary)));
            }
        }

        // If we're at the end and on a new line, show cursor on empty line
        if notes_input.active && notes_input.cursor_line >= lines.len() {
            let cursor_line_in_view = notes_input.cursor_line - start_line;
            if cursor_line_in_view < visible_height as usize {
                // Pad with empty lines if needed
                while display_lines.len() <= cursor_line_in_view {
                    display_lines.push(Line::from(""));
                }
                if display_lines.len() == cursor_line_in_view {
                    display_lines.push(Line::from("█".fg(text_primary)));
                }
            }
        }

        let content_paragraph = Paragraph::new(display_lines)
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(background));

        f.render_widget(content_paragraph, content_area);

        // Add scroll indicator if content is scrollable
        if total_lines > visible_height as usize {
            let scroll_indicator = format!("({}/{})", scroll_offset + 1, total_lines);
            let indicator_area = Rect {
                x: notes_area.x + notes_area.width - scroll_indicator.len() as u16 - 2,
                y: notes_area.y,
                width: scroll_indicator.len() as u16 + 1,
                height: 1,
            };
            let indicator_widget =
                Paragraph::new(scroll_indicator).style(Style::default().fg(text_secondary));
            f.render_widget(indicator_widget, indicator_area);
        }
    } else {
        // Show read-only notes - split by paragraphs
        let mut notes_lines = vec![
            Line::from(vec!["NOTES (N to edit): ".fg(text_secondary)]),
            Line::from(""),
        ];

        // Split notes by single newlines to preserve all line breaks
        let lines: Vec<&str> = todo.notes.split('\n').collect();
        for line in lines.iter() {
            notes_lines.push(Line::from(line.fg(text_primary)));
        }

        // Calculate visible area for read-only mode
        let notes_block = Block::default()
            .title(" Notes ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(background).fg(text_primary))
            .padding(Padding::new(1, 1, 1, 1));

        let inner_area = notes_block.inner(notes_area);
        let visible_height = inner_area.height;

        // Apply scrolling to read-only notes
        let total_lines = notes_lines.len();
        let max_scroll = if total_lines > visible_height as usize {
            total_lines - visible_height as usize
        } else {
            0
        };
        let scroll_offset = (notes_scroll_offset as usize).min(max_scroll);

        // Get visible lines
        let start_line = scroll_offset;
        let end_line = (start_line + visible_height as usize).min(total_lines);
        let visible_lines = if start_line < notes_lines.len() {
            notes_lines[start_line..end_line].to_vec()
        } else {
            Vec::new()
        };

        let notes_paragraph = Paragraph::new(visible_lines)
            .wrap(Wrap { trim: true })
            .block(notes_block);

        f.render_widget(notes_paragraph, notes_area);

        // Add scroll indicator for read-only mode if content is scrollable
        if total_lines > visible_height as usize {
            let scroll_indicator = format!("({}/{})", scroll_offset + 1, total_lines);
            let indicator_area = Rect {
                x: notes_area.x + notes_area.width - scroll_indicator.len() as u16 - 2,
                y: notes_area.y,
                width: scroll_indicator.len() as u16 + 1,
                height: 1,
            };
            let indicator_widget =
                Paragraph::new(scroll_indicator).style(Style::default().fg(text_secondary));
            f.render_widget(indicator_widget, indicator_area);
        }
    }

    // Create a list for subtasks with better spacing
    let subtask_items: Vec<ListItem> = todo
        .subtasks
        .iter()
        .enumerate()
        .map(|(index, subtask)| {
            let line = Line::from(vec![
                Span::styled(
                    format!("{}. ", index + 1),
                    Style::default().fg(Color::Rgb(180, 140, 220)),
                ),
                if subtask.status == "Done" || subtask.status == "Completed" {
                    Span::styled(
                        subtask.text.as_str(),
                        Style::default()
                            .fg(Color::Rgb(120, 220, 150))
                            .add_modifier(Modifier::CROSSED_OUT),
                    )
                } else {
                    Span::styled(subtask.text.as_str(), Style::default().fg(Color::Red))
                },
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = format!(" Subtasks #{} ", todo.subtasks.len());
    let subtask_list = List::new(subtask_items)
        .block(
            Block::default()
                .title(title)
                .fg(Color::Rgb(180, 140, 220))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border).add_modifier(Modifier::BOLD))
                .padding(Padding::new(2, 2, 1, 1))
                .style(Style::default().bg(background).fg(text_primary)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(80, 40, 120)) // Dark purple background for selection
                .add_modifier(Modifier::BOLD),
        )
        // .highlight_symbol("|")
        .repeat_highlight_symbol(true);

    // Render subtasks in the bottom-left with proper spacing
    f.render_stateful_widget(subtask_list, left_layout[2], state);
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

    let area = centered_rect(45, 25, area);
    f.render_widget(block, area);

    let inner_area = area.inner(Margin {
        horizontal: 3,
        vertical: 2,
    });

    let text = vec![
        Line::from(""),
        Line::from("Are you sure you want to delete this item?".fg(text_primary)),
        Line::from(""),
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
        Line::from(""),
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

    f.render_widget(paragraph, inner_area);
}

// Status change confirmation
pub fn draw_priority_modal(f: &mut Frame, area: Rect) {
    // Purple-themed delete confirmation
    let background = Color::Rgb(30, 15, 35);
    let border = Color::Rgb(200, 100, 220);
    let text_primary = Color::Rgb(230, 220, 240);
    let text_secondary = Color::Rgb(200, 180, 220);

    // Calculate dynamic size (45% of width, 30% of height)
    let modal_area = dynamic_rect(45, 30, area);

    let block = Block::default()
        .title(" Priority Change ")
        .borders(Borders::ALL)
        .style(Style::default().bg(background))
        .border_style(Style::default().fg(border).add_modifier(Modifier::BOLD));

    f.render_widget(block, modal_area);

    // Inner area with better padding
    let inner_area = modal_area.inner(Margin {
        horizontal: 3,
        vertical: 2,
    });

    let text = vec![
        Line::from(""),
        Line::from("Set priority for this TODO".fg(text_primary)),
        Line::from(""),
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
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "M",
                Style::default()
                    .fg(Color::Rgb(220, 180, 100))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(": Medium priority".fg(text_secondary)),
        ]),
        Line::from(""),
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
//
// MAIN MODAL MENU
pub fn draw_main_menu_modal(f: &mut Frame, area: Rect) {
    // Theme colors
    let background = Color::Rgb(30, 15, 35);
    let border_color = Color::Rgb(200, 100, 220);
    let text_primary = Color::Rgb(230, 220, 240);
    let text_secondary = Color::Rgb(200, 180, 220);
    let key_color = Color::Rgb(220, 180, 100);

    // Modal dimensions with better sizing
    let modal_area = dynamic_rect(85, 75, area);

    // Main block for the modal
    let block = Block::default()
        .title(" VoiDo Menu ")
        .borders(Borders::ALL)
        .style(Style::default().bg(background))
        .border_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(block, modal_area);

    // Inner layout for content with better spacing
    let inner_area = modal_area.inner(Margin {
        horizontal: 5,
        vertical: 3,
    });

    // Keybindings data
    let keybindings = vec![
        ("Up/Down", "Navigate through the list of TODOs"),
        ("Enter", "Show detailed view of the selected TODO"),
        ("Delete / x", "Delete the selected TODO"),
        ("d", "Mark the selected TODO as 'Done'"),
        ("p", "Mark the selected TODO as 'Pending'"),
        ("o", "Mark the selected TODO as 'Ongoing'"),
        ("P", "Change the priority of the selected TODO"),
        ("M", "Toggle this main menu"),
        ("q", "Quit the application"),
        ("A", "Add a new TODO"),
        ("E", "Export all TODOs to an Excel file"),
        ("Y", "Confirm an action (e.g., deletion)"),
        ("N", "Cancel an action"),
    ];

    // Create rows for the table
    let rows: Vec<Row> = keybindings
        .iter()
        .map(|(key, desc)| {
            Row::new(vec![
                Span::styled(
                    *key,
                    Style::default().fg(key_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(*desc, Style::default().fg(text_secondary)),
            ])
        })
        .collect();

    // Create the table with better spacing
    let table = Table::new(
        rows,
        [
            // Constraint for key column with more space
            Constraint::Length(15),
            // Constraint for description column
            Constraint::Fill(1),
        ],
    )
    .block(
        Block::default()
            .title("Keybindings")
            .borders(Borders::NONE)
            .style(Style::default().fg(text_primary)),
    )
    .column_spacing(5);

    // Render the table
    f.render_widget(table, inner_area);
}
