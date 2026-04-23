use ratatui::style::{Color, Modifier, Style};

pub const ACCENT: Color = Color::Cyan;
pub const ACCENT_DIM: Color = Color::DarkGray;
pub const OK: Color = Color::Green;
pub const WARN: Color = Color::Yellow;
pub const ERR: Color = Color::Red;
pub const FG: Color = Color::White;
pub const BG: Color = Color::Reset;
pub const BG_HIGHLIGHT: Color = Color::DarkGray;

pub fn focused_border() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn unfocused_border() -> Style {
    Style::default().fg(ACCENT_DIM)
}

pub fn selected_row() -> Style {
    Style::default()
        .bg(BG_HIGHLIGHT)
        .add_modifier(Modifier::BOLD)
}

pub fn dim() -> Style {
    Style::default().fg(ACCENT_DIM)
}

pub fn title() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn ok_style() -> Style {
    Style::default().fg(OK)
}

pub fn err_style() -> Style {
    Style::default().fg(ERR)
}

pub fn warn_style() -> Style {
    Style::default().fg(WARN)
}
