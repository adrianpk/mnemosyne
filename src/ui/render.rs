use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{App, AppMode};
use crate::document::index_label;
use super::widgets::render_function_bar;

pub fn draw(frame: &mut Frame, app: &App) {
    let main_layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(frame.area());

    render_function_bar(frame, main_layout[0]);

    let panels = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[1]);

    draw_conversation_panel(frame, app, &panels[0]);
    draw_document_panel(frame, app, &panels[1]);
}

fn draw_conversation_panel(frame: &mut Frame, app: &App, area: &Rect) {
    use crate::app::Role;

    let mut lines: Vec<Line> = Vec::new();
    for msg in &app.conversation {
        let (prefix, style) = match msg.role {
            Role::User => ("> ", Style::default().fg(Color::Cyan)),
            Role::Assistant => ("", Style::default().fg(Color::White)),
        };
        lines.push(Line::from(Span::styled(format!("{}{}", prefix, msg.content), style)));
        lines.push(Line::from(""));
    }

    // Show input or review hint depending on mode
    match &app.mode {
        AppMode::Normal => {
            let input_style = Style::default().fg(Color::Cyan);
            lines.push(Line::from(Span::styled(format!("> {}_", app.input), input_style)));
        }
        AppMode::Review { .. } => {
            let hint_style = Style::default().fg(Color::Yellow);
            lines.push(Line::from(Span::styled(
                "[Enter] Accept  [Esc] Discard",
                hint_style,
            )));
        }
    }

    let conversation = Paragraph::new(lines)
        .block(Block::default().title("Conversation").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(conversation, *area);
}

fn draw_document_panel(frame: &mut Frame, app: &App, area: &Rect) {
    let review_info = match &app.mode {
        AppMode::Review { suggestion, paragraph_index } => Some((suggestion, *paragraph_index)),
        AppMode::Normal => None,
    };

    let doc_lines: Vec<Line> = app
        .document
        .paragraphs
        .iter()
        .enumerate()
        .flat_map(|(i, p)| {
            let index_style = Style::default().add_modifier(Modifier::DIM);

            // Check if this paragraph is being reviewed
            if let Some((suggestion, review_idx)) = &review_info {
                if i == *review_idx {
                    // Show diff view (Tokyo Night palette)
                    let removed_style = Style::default().fg(Color::Rgb(224, 175, 104)); // yellow
                    let added_style = Style::default().fg(Color::Rgb(187, 154, 247));   // purple

                    let label = Span::styled(format!("[{}] ", index_label(i)), index_style);
                    let removed_line = Line::from(vec![
                        label.clone(),
                        Span::styled(format!("- {}", suggestion.original), removed_style),
                    ]);
                    let added_line = Line::from(vec![
                        Span::styled("    ", index_style), // spacing to align with label
                        Span::styled(format!("+ {}", suggestion.replacement), added_style),
                    ]);

                    return vec![removed_line, added_line, Line::from("")];
                }
            }

            // Normal view
            let style = if i == app.document.selected && review_info.is_none() {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            let line = Line::from(vec![
                Span::styled(format!("[{}] ", index_label(i)), index_style),
                Span::styled(p.clone(), style),
            ]);
            vec![line, Line::from("")]
        })
        .collect();

    let scroll = calculate_scroll(app, area);

    let panel = Paragraph::new(doc_lines)
        .block(Block::default().title("Document").borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(panel, *area);
}

fn calculate_scroll(app: &App, area: &Rect) -> u16 {
    let panel_width = area.width.saturating_sub(2) as usize;
    let panel_height = area.height.saturating_sub(2);

    let mut line_count = 0u16;
    let mut selected_start = 0u16;

    for (i, p) in app.document.paragraphs.iter().enumerate() {
        if i == app.document.selected {
            selected_start = line_count;
        }
        let label = format!("[{}] ", index_label(i));
        let text_len = label.len() + p.len();
        let lines = if panel_width > 0 {
            ((text_len / panel_width) + 1) as u16
        } else {
            1
        };
        line_count += lines + 1;
    }

    let selected_label = format!("[{}] ", index_label(app.document.selected));
    let selected_text_len =
        selected_label.len() + app.document.paragraphs[app.document.selected].len();
    let selected_lines = if panel_width > 0 {
        ((selected_text_len / panel_width) + 1) as u16
    } else {
        1
    };
    let selected_end = selected_start + selected_lines + 1;

    if selected_end > panel_height {
        selected_start
    } else {
        0
    }
}
