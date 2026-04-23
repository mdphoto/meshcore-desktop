use crate::action::ToastLevel;
use crate::state::Toast;
use crate::theme;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(frame: &mut Frame, area: Rect, toasts: &[&Toast]) {
    if toasts.is_empty() {
        return;
    }
    let height = (toasts.len() as u16 + 2).min(area.height);
    let width = 60.min(area.width);
    let x = area.x + area.width.saturating_sub(width) - 1;
    let y = area.y + 1;
    let rect = Rect {
        x,
        y,
        width,
        height,
    };

    let lines: Vec<Line> = toasts
        .iter()
        .map(|t| {
            let (prefix, style) = match t.level {
                ToastLevel::Info => ("i", Style::default().fg(theme::ACCENT)),
                ToastLevel::Success => ("✓", theme::ok_style()),
                ToastLevel::Warn => ("!", theme::warn_style()),
                ToastLevel::Error => ("✗", theme::err_style()),
            };
            Line::from(vec![
                Span::styled(format!(" {} ", prefix), style),
                Span::raw(t.message.clone()),
            ])
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::dim())
        .title(" messages ");
    let para = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);
    frame.render_widget(ratatui::widgets::Clear, rect);
    frame.render_widget(para, rect);
}
