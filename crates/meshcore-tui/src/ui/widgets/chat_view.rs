use crate::theme;
use crate::util::format::{relative_time, strip_sender_prefix};
use meshcore_storage::models::StoredMessage;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    messages: &[StoredMessage],
    fully_loaded: bool,
    scroll_offset: u16,
    focused: bool,
) {
    // Le titre (nom du canal / DM) est toujours affiché en accent cyan bold
    // indépendamment du focus de la zone, pour que l'utilisateur repère toujours
    // d'un coup d'œil dans quelle conversation il se trouve.
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(title.to_string(), theme::title()))
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    if messages.is_empty() {
        let body = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Aucun message dans cette conversation.",
                theme::dim(),
            )),
        ])
        .block(block);
        frame.render_widget(body, area);
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(messages.len() * 2 + 1);
    if fully_loaded {
        lines.push(Line::from(Span::styled(
            " ── début de la conversation ── ",
            theme::dim(),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            " ↑ PgUp pour charger les messages plus anciens",
            theme::dim(),
        )));
    }
    for msg in messages {
        lines.push(message_line(msg));
    }

    let total = lines.len() as u16;
    let inner_height = area.height.saturating_sub(2);
    // Auto-scroll « sticky-bottom » : si scroll_offset == 0 on ancre en bas
    let display_scroll = if scroll_offset == 0 {
        total.saturating_sub(inner_height)
    } else {
        total.saturating_sub(inner_height).saturating_sub(scroll_offset)
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((display_scroll, 0));

    frame.render_widget(paragraph, area);

    if total > inner_height {
        let mut scrollbar_state = ScrollbarState::new(total.saturating_sub(inner_height) as usize)
            .position(display_scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        frame.render_stateful_widget(
            scrollbar,
            Rect {
                x: area.x + area.width.saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: inner_height,
            },
            &mut scrollbar_state,
        );
    }
}

fn message_line(msg: &StoredMessage) -> Line<'static> {
    let outgoing = msg.direction == "outgoing";
    let (prefix_style, status_icon) = if outgoing {
        let icon = match msg.status.as_str() {
            "pending" => Span::styled(" ⏳", theme::dim()),
            "sent" => Span::styled(" ✓", theme::warn_style()),
            "delivered" => Span::styled(" ✓✓", theme::ok_style()),
            "failed" => Span::styled(" ✗", theme::err_style()),
            _ => Span::raw(""),
        };
        (Style::default().fg(theme::ACCENT), icon)
    } else {
        (theme::ok_style(), Span::raw(""))
    };

    let prefix = if outgoing {
        "moi".to_string()
    } else if !msg.sender_name.is_empty() {
        msg.sender_name.clone()
    } else if let Some(ref pk) = msg.sender_pubkey {
        pk[..pk.len().min(12)].to_string()
    } else {
        "?".to_string()
    };

    let ts = relative_time(&msg.timestamp);
    // Retire le préfixe « Nom: » du texte si on l'a déjà dans sender_name
    // (évite l'affichage dupliqué)
    let clean_text = strip_sender_prefix(&msg.text, &msg.sender_name);

    Line::from(vec![
        Span::styled(format!(" [{}] ", ts), theme::dim()),
        Span::styled(format!("{}: ", prefix), prefix_style),
        Span::raw(clean_text.to_string()),
        status_icon,
    ])
}
