use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct MainAreas {
    pub tabs: Rect,
    pub body: Rect,
    pub status: Rect,
}

pub fn split_main(area: Rect) -> MainAreas {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(area);
    MainAreas {
        tabs: chunks[0],
        body: chunks[1],
        status: chunks[2],
    }
}
