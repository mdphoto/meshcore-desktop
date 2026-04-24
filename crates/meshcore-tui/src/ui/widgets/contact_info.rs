use crate::theme;
use crate::util::format::{
    format_contact_flags, format_path_len, format_unix_timestamp, node_type_label,
};
use meshcore_storage::models::StoredContact;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

/// Rend une modale plein-écran avec les infos publiques d'un contact.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    contact: &StoredContact,
    extra_hints: &[&str],
) {
    let rect = centered_rect(68, 70, area);
    let title = format!(" Info contact — {} ", contact.name);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::focused_border())
        .title(title);

    let mut body: Vec<Line> = vec![
        Line::from(""),
        kv("Type", node_type_label(contact.node_type)),
        kv("Nom", &contact.name),
        kv("Pubkey", &contact.public_key),
        Line::from(Span::styled(
            "    [F5] copier la pubkey dans le presse-papier",
            theme::dim(),
        )),
        Line::from(""),
    ];

    // GPS
    if contact.lat != 0.0 || contact.lon != 0.0 {
        body.push(kv(
            "GPS",
            &format!("{:.5}, {:.5}", contact.lat, contact.lon),
        ));
        body.push(Line::from(vec![
            Span::styled(format!(" {:<12} ", ""), theme::dim()),
            Span::styled(
                format!(
                    "https://www.openstreetmap.org/?mlat={:.5}&mlon={:.5}&zoom=14",
                    contact.lat, contact.lon
                ),
                theme::dim(),
            ),
        ]));
    } else {
        body.push(kv("GPS", "(non diffusée)"));
    }

    // Last seen (timestamp Unix stocké en string)
    body.push(kv(
        "Dernière vue",
        &format_unix_timestamp(&contact.last_seen),
    ));

    // Chemin : -1 = flood, 0 = direct, N > 0 = N hop(s)
    let path_line = if contact.path.is_empty() || contact.path_len <= 0 {
        format_path_len(contact.path_len)
    } else {
        format!(
            "{}  ({})",
            format_path_len(contact.path_len),
            contact
                .path
                .iter()
                .take(contact.path_len as usize)
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(":")
        )
    };
    body.push(kv("Chemin", &path_line));

    // Flags (bitfield décodé)
    body.push(kv("Flags", &format_contact_flags(contact.flags)));

    // Favori
    body.push(kv(
        "Favori",
        if contact.is_favorite {
            "★ oui"
        } else {
            "non"
        },
    ));

    // Group
    body.push(kv(
        "Groupe",
        contact.group_name.as_deref().unwrap_or("(aucun)"),
    ));

    body.push(Line::from(""));
    for hint in extra_hints {
        body.push(Line::from(Span::styled(
            format!("  {}", hint),
            theme::dim(),
        )));
    }

    body.push(Line::from(""));
    body.push(Line::from(vec![
        Span::styled("[F5]", theme::title()),
        Span::raw(" copier pubkey  "),
        Span::styled("[Esc]", theme::title()),
        Span::raw(" fermer"),
    ]));

    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(body)
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false }),
        rect,
    );
}

fn kv(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {:<13} ", label), theme::title()),
        Span::raw(value.to_string()),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}
