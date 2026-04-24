use crate::app::App;
use crate::state::FocusTarget;
use crate::state::chat::{ChatFocus, ConversationId, ConversationKind};
use crate::theme;
use crate::util::i18n::t;
use crate::util::unicode::truncate;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(0)])
        .split(area);

    render_conversations(frame, chunks[0], app);
    render_right_panel(frame, chunks[1], app);
}

fn render_conversations(frame: &mut Frame, area: Rect, app: &App) {
    let focused = matches!(app.ui.focus, FocusTarget::ChatList);
    let summaries = app.chat_conversation_summaries();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ({}) ", t("chat.conversations"), summaries.len()))
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    if summaries.is_empty() {
        let body = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("chat.empty")),
                theme::dim(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("chat.empty_hint")),
                theme::dim(),
            )),
        ])
        .block(block);
        frame.render_widget(body, area);
        return;
    }

    // Construction avec sections visuelles (Canaux / Rooms / Messages directs).
    // On trace une map summary_idx → display_idx pour convertir la sélection logique
    // (qui vit dans conversations_list_state et indexe `summaries`) vers la position
    // affichée dans la List ratatui (qui inclut les headers).
    let mut items: Vec<ListItem> = Vec::new();
    let mut summary_to_display: Vec<usize> = Vec::with_capacity(summaries.len());
    let mut last_kind: Option<ConversationKind> = None;
    for s in &summaries {
        if last_kind != Some(s.kind) {
            if last_kind.is_some() {
                items.push(ListItem::new(Line::from(""))); // ligne vide de séparation
            }
            let header = match s.kind {
                ConversationKind::Channel => t("chat.section.channels"),
                ConversationKind::Room => t("chat.section.rooms"),
                ConversationKind::Dm => t("chat.section.dms"),
            };
            items.push(ListItem::new(Line::from(Span::styled(
                header,
                theme::title(),
            ))));
            last_kind = Some(s.kind);
        }
        summary_to_display.push(items.len()); // position de la prochaine conv dans items
        let marker = match s.kind {
            ConversationKind::Channel => "#",
            ConversationKind::Room => "R",
            ConversationKind::Dm => "·",
        };
        let unread = if s.unread > 0 {
            format!(" ({})", s.unread)
        } else {
            String::new()
        };
        // État de login d'une room → couleur du nom :
        //   - pending  → orange (warn_style)
        //   - success  → vert (ok_style)
        //   - failed   → rouge (err_style)
        //   - non-tenté → normal
        let name_style = if let (ConversationKind::Room, ConversationId::Dm(pk)) =
            (s.kind, &s.id)
        {
            let is_pending = app
                .ui
                .room_login_pending
                .as_ref()
                .is_some_and(|(ppk, _, _)| ppk == pk);
            if is_pending {
                theme::warn_style()
            } else if app.rooms_logged_in.contains(pk) {
                theme::ok_style()
            } else if app.rooms_login_failed.contains(pk) {
                theme::err_style()
            } else {
                ratatui::style::Style::default()
            }
        } else {
            ratatui::style::Style::default()
        };
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::raw(format!(" {} ", marker)),
            Span::styled(truncate(&s.display_name, 20), name_style),
            Span::styled(unread, theme::warn_style()),
        ]));
        if let Some(last) = &s.last_message {
            lines.push(Line::from(Span::styled(
                format!("    {}", truncate(last, 22)),
                theme::dim(),
            )));
        }
        items.push(ListItem::new(lines));
    }

    // Conversion : l'index stocké dans conversations_list_state est un index summary,
    // on trouve la position display correspondante pour que ratatui highlight la bonne ligne
    let selected_summary = app.chat_ui.conversations_list_state.selected();
    let display_selected = selected_summary.and_then(|i| summary_to_display.get(i).copied());
    let mut list_state = ratatui::widgets::ListState::default().with_selected(display_selected);

    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selected_row())
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_right_panel(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3), Constraint::Length(1)])
        .split(area);

    let Some(active) = &app.chat_ui.active else {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", t("tab.chat").trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ')))
            .border_style(theme::unfocused_border());
        let body = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("chat.welcome")),
                theme::dim(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("chat.welcome_hint")),
                theme::dim(),
            )),
        ])
        .block(block);
        frame.render_widget(body, chunks[0]);
        render_hints(frame, chunks[2], app);
        return;
    };

    let title = match active {
        ConversationId::Dm(pk) => {
            let name = app
                .contacts
                .iter()
                .find(|c| c.public_key == *pk)
                .map(|c| c.name.as_str())
                .unwrap_or("contact");
            format!(" DM · {} ", name)
        }
        ConversationId::Channel(idx) => {
            let name = app
                .channels
                .iter()
                .find(|c| c.idx == *idx)
                .map(|c| c.name.as_str())
                .unwrap_or("?");
            format!(" #{} · {} ", idx, name)
        }
    };

    let messages = app.chat_ui.active_messages().cloned().unwrap_or_default();
    let fully_loaded = app
        .chat_ui
        .fully_loaded
        .get(active)
        .copied()
        .unwrap_or(false);

    let hist_focused = matches!(app.ui.focus, FocusTarget::ChatHistory);
    crate::ui::widgets::chat_view::render(
        frame,
        chunks[0],
        &title,
        &messages,
        fully_loaded,
        app.chat_ui.active_scroll(),
        hist_focused,
    );

    let input_focused = matches!(app.ui.focus, FocusTarget::ChatInput);
    let input_title = format!(" {} ", t("chat.msg.placeholder"));
    crate::ui::widgets::input_box::render(
        frame,
        chunks[1],
        &input_title,
        &app.chat_ui.input,
        input_focused,
    );

    // Popup @mention rendu au-dessus de l'input si ouvert
    if let Some(mention) = app.chat_ui.mention.as_ref() {
        crate::ui::widgets::mention_popup::render(frame, chunks[1], mention);
    }

    render_hints(frame, chunks[2], app);
}

fn render_hints(frame: &mut Frame, area: Rect, app: &App) {
    let focus_hint = match app.chat_ui.focus {
        ChatFocus::List => t("chat.focus.list"),
        ChatFocus::History => t("chat.focus.history"),
        ChatFocus::Input => t("chat.focus.input"),
    };
    let spans = vec![
        Span::styled("  ", theme::dim()),
        Span::styled("Tab", theme::title()),
        Span::styled(
            format!(" {} ({})  ·  ", t("chat.focus_label"), focus_hint),
            theme::dim(),
        ),
        Span::styled("PgUp", theme::title()),
        Span::styled(format!(" {}  ·  ", t("chat.hint_older")), theme::dim()),
        Span::styled("?", theme::title()),
        Span::styled(format!(" {}", t("statusbar.help")), theme::dim()),
    ];
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
