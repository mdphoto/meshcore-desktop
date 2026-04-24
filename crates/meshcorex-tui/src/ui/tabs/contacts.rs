use crate::actions::contacts::ContactRow;
use crate::app::App;
use crate::state::FocusTarget;
use crate::theme;
use crate::util::format::{node_type_icon, node_type_label, node_type_plural, short_pubkey};
use crate::util::i18n::t;
use crate::util::unicode::truncate;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let focused = matches!(app.ui.focus, FocusTarget::ContactsList);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let sync_hint = if app.ui.contacts_syncing {
        let elapsed = app
            .ui
            .contacts_sync_started_at
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0);
        format!(" · {} sync {}s", crate::util::format::spinner_frame(), elapsed)
    } else {
        String::new()
    };
    let title = format!(
        " {} ({}) · {} : {}{} · {} ",
        t("contacts.title"),
        app.contacts.len(),
        t("contacts.sort_label"),
        app.ui.contacts_sort.label(),
        sync_hint,
        t("contacts.hints"),
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    if app.contact_rows.is_empty() {
        let body = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("contacts.empty")),
                theme::dim(),
            )),
        ])
        .block(block);
        frame.render_widget(body, chunks[0]);
        render_hints(frame, chunks[1]);
        return;
    }

    let items: Vec<ListItem> = app
        .contact_rows
        .iter()
        .map(|row| match row {
            ContactRow::Header { node_type, count } => {
                let collapsed = app.ui.contacts_collapsed_groups.contains(node_type);
                let chevron = if collapsed { "▶" } else { "▼" };
                let text = format!(" {} {} ({})", chevron, node_type_plural(*node_type), count);
                ListItem::new(Line::from(Span::styled(text, theme::title())))
            }
            ContactRow::Contact(idx) => {
                let c = match app.contacts.get(*idx) {
                    Some(c) => c,
                    None => return ListItem::new(Line::from("")),
                };
                let fav = if c.is_favorite { "★" } else { " " };
                let label = format!(
                    "   {} {} {} {}",
                    fav,
                    node_type_icon(c.node_type),
                    truncate(&c.name, 24),
                    short_pubkey(&c.public_key),
                );
                let mut spans = vec![Span::raw(label)];
                if c.node_type == 2 {
                    spans.push(Span::styled("  [R]", theme::warn_style()));
                }
                spans.push(Span::styled(
                    format!("  ({})", node_type_label(c.node_type)),
                    theme::dim(),
                ));
                ListItem::new(Line::from(spans))
            }
        })
        .collect();

    let mut list_state = app.contacts_list_state.clone();
    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selected_row())
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, chunks[0], &mut list_state);
    render_hints(frame, chunks[1]);
}

fn render_hints(frame: &mut Frame, area: Rect) {
    let spans = vec![
        Span::styled("  ", theme::dim()),
        Span::styled("Enter", theme::title()),
        Span::styled(format!(" {}  ·  ", t("contacts.hint_fold")), theme::dim()),
        Span::styled("?", theme::title()),
        Span::styled(format!(" {}", t("contacts.hint_help")), theme::dim()),
    ];
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
