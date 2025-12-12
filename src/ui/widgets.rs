use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::Paragraph,
};

pub struct FunctionKey {
    pub key: &'static str,
    pub label: &'static str,
}

pub const FUNCTION_KEYS: &[FunctionKey] = &[
    FunctionKey { key: " F1 ", label: " Help " },
    FunctionKey { key: " F2 ", label: " Open " },
    FunctionKey { key: " F3 ", label: " Crit " },
    FunctionKey { key: " F4 ", label: " Summ " },
    FunctionKey { key: " F5 ", label: " Styl " },
    FunctionKey { key: " F6 ", label: " Gram " },
    FunctionKey { key: " F7 ", label: " Reph " },
    FunctionKey { key: " F8 ", label: " Thes " },
    FunctionKey { key: " F9 ", label: " Over " },
    FunctionKey { key: " F10 ", label: " Echo " },
    FunctionKey { key: " F11 ", label: " Exit " },
    FunctionKey { key: " F12 ", label: " Conf " },
];

pub fn render_function_bar(frame: &mut Frame, area: Rect) {
    let key_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let label_style = Style::default()
        .fg(Color::White)
        .bg(Color::DarkGray);

    let constraints: Vec<Constraint> = FUNCTION_KEYS
        .iter()
        .map(|_| Constraint::Ratio(1, FUNCTION_KEYS.len() as u32))
        .collect();

    let chunks = Layout::horizontal(constraints).split(area);

    for (i, fk) in FUNCTION_KEYS.iter().enumerate() {
        let chunk_width = chunks[i].width as usize;
        let key_len = fk.key.len();
        let available_for_label = chunk_width.saturating_sub(key_len);

        let label = if fk.label.len() > available_for_label {
            &fk.label[..available_for_label]
        } else {
            fk.label
        };

        let key_span = Span::styled(fk.key, key_style);
        let label_span = Span::styled(label, label_style);

        let key_widget = Paragraph::new(key_span);
        let label_widget = Paragraph::new(label_span);

        let inner = Layout::horizontal([
            Constraint::Length(key_len as u16),
            Constraint::Min(0),
        ])
        .split(chunks[i]);

        frame.render_widget(key_widget, inner[0]);
        frame.render_widget(label_widget, inner[1]);
    }
}
