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

    // Largeur utile (sans les bordures)
    let inner_width = area.width.saturating_sub(2) as usize;
    // Scroll horizontal calculé par tui-input pour garder le curseur visible
    let scroll = input.visual_scroll(inner_width);
    let value = input.value();

    // Partie du texte réellement affichée après scroll
    let visible: String = value.chars().skip(scroll).take(inner_width).collect();

    frame.render_widget(
        Paragraph::new(Line::from(Span::raw(visible))).block(block),
        area,
    );

    // Vrai curseur terminal à la position exacte du curseur interne tui-input
    // (corrige le bug « _ affiché à la fin alors que le curseur interne est ailleurs »
    //  et rend Backspace / ← / → / Home / End cohérents avec l'affichage)
    if focused {
        let cursor_in_view = input.visual_cursor().saturating_sub(scroll);
        let cursor_x = area.x + 1 + cursor_in_view as u16;
        let cursor_y = area.y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}
