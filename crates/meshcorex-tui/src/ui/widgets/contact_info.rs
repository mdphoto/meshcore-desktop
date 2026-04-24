use crate::theme;
use crate::util::format::{
    format_contact_flags, format_path_len, format_unix_timestamp, node_type_label,
};
use crate::util::i18n::t;
use meshcorex_storage::models::StoredContact;
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
    let title = format!(" {} — {} ", t("contact.info.title"), contact.name);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::focused_border())
        .title(title);

    let mut body: Vec<Line> = vec![
        Line::from(""),
        kv(&t("contact.info.type"), node_type_label(contact.node_type)),
        kv(&t("contact.info.name"), &contact.name),
        kv(&t("contact.info.pubkey"), &contact.public_key),
        Line::from(Span::styled(
            format!("    {}", t("contact.info.copy_hint")),
            theme::dim(),
        )),
        Line::from(""),
    ];

    // GPS
    if contact.lat != 0.0 || contact.lon != 0.0 {
        body.push(kv(
            &t("contact.info.gps"),
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
        body.push(kv(&t("contact.info.gps"), &t("contact.info.gps_none")));
    }

    // Last seen (timestamp Unix stocké en string)
    body.push(kv(
        &t("contact.info.last_seen"),
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
    body.push(kv(&t("contact.info.path"), &path_line));

    // Flags (bitfield décodé)
    body.push(kv(&t("contact.info.flags"), &format_contact_flags(contact.flags)));

    // Favori
    body.push(kv(
        &t("contact.info.favorite"),
        &if contact.is_favorite {
            t("contact.info.favorite_yes")
        } else {
            t("contact.info.favorite_no")
        },
    ));

    // Group
    let group_none = t("contact.info.group_none");
    body.push(kv(
        &t("contact.info.group"),
        contact.group_name.as_deref().unwrap_or(&group_none),
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
        Span::raw(format!(" {}  ", t("contact.info.action_copy"))),
        Span::styled("[Esc]", theme::title()),
        Span::raw(format!(" {}", t("contact.info.action_close"))),
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
