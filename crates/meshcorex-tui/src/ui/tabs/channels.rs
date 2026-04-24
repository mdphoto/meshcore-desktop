use crate::app::App;
use crate::state::FocusTarget;
use crate::theme;
use crate::util::i18n::t;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let focused = matches!(app.ui.focus, FocusTarget::ChannelsList);
    let total = app.channels.len();
    let title = format!(
        " {} ({}) — {} ",
        t("channels.title"),
        total,
        t("channels.hints"),
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    if app.channels.is_empty() {
        let body = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("channels.empty")),
                theme::dim(),
            )),
        ])
        .block(block);
        frame.render_widget(body, area);
        return;
    }

    let items: Vec<ListItem> = app
        .channels
        .iter()
        .map(|c| {
            let unread = if c.unread_count > 0 {
                format!(" ({})", c.unread_count)
            } else {
                String::new()
            };
            let notif = if c.notifications_enabled { "🔔" } else { "🔕" };
            let scope = app
                .channel_scopes
                .get(&c.idx)
                .map(|s| format!("  scope:{}", s))
                .unwrap_or_default();
            ListItem::new(Line::from(vec![
                Span::raw(format!(" #{:>2}  ", c.idx)),
                Span::raw(c.name.clone()),
                Span::styled(unread, theme::warn_style()),
                Span::raw("  "),
                Span::styled(notif, theme::dim()),
                Span::styled(format!("  [{}]", c.channel_type), theme::dim()),
                Span::styled(scope, theme::warn_style()),
            ]))
        })
        .collect();

    let mut list_state = app.channels_list_state.clone();
    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selected_row())
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut list_state);
}
