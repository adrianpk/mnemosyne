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

pub fn draw(frame: &mut Frame, app: &App<'_>) {
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

fn draw_conversation_panel(frame: &mut Frame, app: &App<'_>, area: &Rect) {
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

    match &app.mode {
        AppMode::Normal => {
            let input_style = Style::default().fg(Color::Cyan);
            if app.input.is_empty() {
                lines.push(Line::from(Span::styled("> _", input_style)));
            } else {
                lines.push(Line::from(Span::styled(format!("> {}_", app.input), input_style)));
            }
        }
        AppMode::Review { .. } => {
            let hint_style = Style::default().fg(Color::Yellow);
            lines.push(Line::from(Span::styled(
                "[Enter] Accept  [Esc] Discard",
                hint_style,
            )));
        }
        AppMode::SelectAlternative { alternatives, .. } => {
            let hint_style = Style::default().fg(Color::Yellow);
            let hint = format!("[1-{}] Apply  [Esc] Cancel", alternatives.len());
            lines.push(Line::from(Span::styled(hint, hint_style)));
        }
        AppMode::Edit { .. } => {
            let hint_style = Style::default().fg(Color::Yellow);
            lines.push(Line::from(Span::styled(
                "Editing — [Ctrl+S] Save  [Esc] Cancel",
                hint_style,
            )));
        }
        AppMode::BrowseRepeats { terms, highlighted } => {
            let hint_style = Style::default().fg(Color::Yellow);
            let label_hint = if terms.len() <= 9 {
                format!("[1-{}]", terms.len())
            } else {
                let last_label = index_label(terms.len() - 1);
                format!("[1-9,a-{}]", last_label)
            };
            let hint = format!(
                "{} Highlight  [Esc] Exit{}",
                label_hint,
                highlighted.as_ref().map(|t| format!("  → '{}'", t)).unwrap_or_default()
            );
            lines.push(Line::from(Span::styled(hint, hint_style)));
        }
        AppMode::MoreMenu => {
            // Clear previous lines and show the More menu
            lines.clear();
            let title_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
            let hint_style = Style::default().fg(Color::Yellow);

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("More Operations", title_style)));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("  (No additional operations yet)", Style::default().fg(Color::DarkGray))));
            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("[q] Close  [F12] Close", hint_style)));
        }
    }

    // Calculate scroll
    let panel_height = area.height.saturating_sub(2) as usize; // minus borders
    let panel_width = area.width.saturating_sub(2) as usize;

    // Estimate total lines (accounting for wrap)
    // Use ceiling division for more accurate wrapping calculation
    let total_lines: usize = lines.iter().map(|line| {
        let len = line.width();
        if panel_width > 0 && len > panel_width {
            // Ceiling division: (len + panel_width - 1) / panel_width
            (len + panel_width - 1) / panel_width
        } else {
            1
        }
    }).sum();

    // Auto-scroll to bottom, but allow manual scroll up via conversation_scroll
    let max_scroll = total_lines.saturating_sub(panel_height);
    let scroll = if app.conversation_scroll == 0 {
        // Auto-scroll: show most recent
        max_scroll as u16
    } else {
        // Manual scroll: offset from bottom
        max_scroll.saturating_sub(app.conversation_scroll) as u16
    };

    let conversation = Paragraph::new(lines)
        .block(Block::default().title("Conversation").borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(conversation, *area);
}

fn draw_document_panel(frame: &mut Frame, app: &App<'_>, area: &Rect) {
    // Check if we're in Edit mode
    if let AppMode::Edit { paragraph_index, original } = &app.mode {
        // Split area: original on top (40%), editor below (60%)
        let chunks = Layout::vertical([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(*area);

        // Draw original text
        let index_style = Style::default().add_modifier(Modifier::DIM);
        let removed_style = Style::default().fg(Color::Rgb(224, 175, 104)); // yellow
        let original_lines = vec![
            Line::from(vec![
                Span::styled(format!("[{}] ", index_label(*paragraph_index)), index_style),
                Span::styled(format!("- {}", original), removed_style),
            ]),
        ];
        let original_panel = Paragraph::new(original_lines)
            .block(Block::default().title("Original").borders(Borders::ALL))
            .wrap(Wrap { trim: false });
        frame.render_widget(original_panel, chunks[0]);

        // Draw editor
        if let Some(editor) = &app.editor {
            frame.render_widget(editor, chunks[1]);
        }
        return;
    }

    let review_info = match &app.mode {
        AppMode::Review { suggestion, paragraph_index } => Some((suggestion, *paragraph_index)),
        _ => None,
    };

    let doc_lines: Vec<Line> = app
        .document
        .paragraphs
        .iter()
        .enumerate()
        .flat_map(|(i, p)| {
            let index_style = Style::default().add_modifier(Modifier::DIM);

            // NOTE: I need to check if this paragraph is being reviewed
            if let Some((suggestion, review_idx)) = &review_info {
                if i == *review_idx {
                    // Show diff view
                    let removed_style = Style::default().fg(Color::Rgb(224, 175, 104)); // yellow
                    let added_style = Style::default().fg(Color::Rgb(187, 154, 247));   // purple

                    let label = Span::styled(format!("[{}] ", index_label(i)), index_style);
                    let removed_line = Line::from(vec![
                        label.clone(),
                        Span::styled(format!("- {}", suggestion.original), removed_style),
                    ]);
                    let added_line = Line::from(vec![
                        Span::styled("    ", index_style), // NOTE: This letme add spacing to align with label
                        Span::styled(format!("+ {}", suggestion.replacement), added_style),
                    ]);

                    return vec![removed_line, added_line, Line::from("")];
                }
            }

            // Normal view
            let base_style = if app.highlight_all {
                // Full-document operation: highlight all paragraphs
                Style::default().add_modifier(Modifier::REVERSED)
            } else if i == app.document.selected && review_info.is_none() {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            // Check if we need to highlight a term
            let spans = if let Some(term) = app.highlighted_term() {
                highlight_term_in_text(p, term, base_style)
            } else {
                vec![Span::styled(p.clone(), base_style)]
            };

            let mut line_spans = vec![Span::styled(format!("[{}] ", index_label(i)), index_style)];
            line_spans.extend(spans);
            let line = Line::from(line_spans);
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

fn calculate_scroll(app: &App<'_>, area: &Rect) -> u16 {
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

/// Split text into spans, highlighting occurrences of the search term
fn highlight_term_in_text<'a>(text: &'a str, term: &str, base_style: Style) -> Vec<Span<'a>> {
    let highlight_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    let term_lower = term.to_lowercase();
    let text_lower = text.to_lowercase();

    let mut spans = Vec::new();
    let mut last_end = 0;

    // Find all occurrences (case-insensitive)
    for (start, _) in text_lower.match_indices(&term_lower) {
        // Add text before the match
        if start > last_end {
            spans.push(Span::styled(&text[last_end..start], base_style));
        }
        // Add the highlighted match (preserve original case)
        let end = start + term.len();
        spans.push(Span::styled(&text[start..end], highlight_style));
        last_end = end;
    }

    // Add remaining text after last match
    if last_end < text.len() {
        spans.push(Span::styled(&text[last_end..], base_style));
    }

    // If no matches, return the whole text with base style
    if spans.is_empty() {
        spans.push(Span::styled(text, base_style));
    }

    spans
}
