use crate::theme;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use tui_input::Input;

pub fn render(frame: &mut Frame, area: Rect, title: &str, input: &Input, focused: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title.to_string())
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    let width = area.width.saturating_sub(2) as usize;
    let scroll = input.visual_scroll(width);
    let value = input.value();
    let visible: String = value
        .chars()
        .skip(scroll)
        .take(width.max(1))
        .collect();

    let line = if focused {
        Line::from(vec![
            Span::raw(visible),
            Span::styled("_", theme::dim()),
        ])
    } else {
        Line::from(Span::raw(visible))
    };
    frame.render_widget(Paragraph::new(line).block(block), area);
}
