use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;
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

    draw_conversation_panel(frame, &panels[0]);
    draw_document_panel(frame, app, &panels[1]);
}

fn draw_conversation_panel(frame: &mut Frame, area: &Rect) {
    let panel = Paragraph::new("AI Assistant")
        .block(Block::default().title("Conversation").borders(Borders::ALL));
    frame.render_widget(panel, *area);
}

fn draw_document_panel(frame: &mut Frame, app: &App, area: &Rect) {
    let doc_lines: Vec<Line> = app
        .document
        .paragraphs
        .iter()
        .enumerate()
        .flat_map(|(i, p)| {
            let style = if i == app.document.selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            let index_style = Style::default().add_modifier(Modifier::DIM);
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
