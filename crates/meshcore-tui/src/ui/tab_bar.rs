use crate::action::Tab;
use crate::theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
};

/// Version récupérée à la compilation depuis le Cargo.toml du crate
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn render(frame: &mut Frame, area: Rect, current: &Tab) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| Line::from(Span::raw(t.title())))
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::dim())
        .title(format!(" MeshCore TUI v{} ", VERSION));

    let tabs = Tabs::new(titles)
        .block(block)
        .select(current.index())
        .style(Style::default().fg(theme::FG))
        .highlight_style(theme::focused_border())
        .divider("│");

    frame.render_widget(tabs, area);
}
