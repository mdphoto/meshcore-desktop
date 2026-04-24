use crate::state::chat::MentionState;
use crate::theme;
use crate::util::i18n::t;
use crate::util::unicode::truncate;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// Rendu d'un popup flottant `@mention` juste au-dessus du champ de saisie chat.
/// L'ancre `input_area` est le Rect de l'input box — le popup est dessiné au-dessus.
pub fn render(frame: &mut Frame, input_area: Rect, mention: &MentionState) {
    let filtered = mention.filtered();

    // Taille : largeur ~40 ou celle de l'input, hauteur = nb_candidats (max 8) + 2 bordures
    let width = 40.min(input_area.width);
    let items_count = filtered.len().clamp(1, 8) as u16;
    let height = items_count + 2;
    let y = input_area.y.saturating_sub(height);
    let popup_area = Rect {
        x: input_area.x,
        y,
        width,
        height,
    };

    let title = if mention.query.is_empty() {
        format!(" @mention ({}) ", filtered.len())
    } else {
        format!(
            " @{} ({} {}) ",
            mention.query,
            filtered.len(),
            t("mention.results"),
        )
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::focused_border())
        .title(title);

    frame.render_widget(Clear, popup_area);

    if filtered.is_empty() {
        let body = Paragraph::new(Line::from(Span::styled(
            format!("  {}", t("mention.empty")),
            theme::dim(),
        )))
        .block(block);
        frame.render_widget(body, popup_area);
        return;
    }

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == mention.selected {
                theme::selected_row()
            } else {
                ratatui::style::Style::default()
            };
            ListItem::new(Line::from(Span::styled(
                format!(" {} ", truncate(name, width.saturating_sub(2) as usize)),
                style,
            )))
        })
        .collect();

    frame.render_widget(List::new(items).block(block), popup_area);
}
