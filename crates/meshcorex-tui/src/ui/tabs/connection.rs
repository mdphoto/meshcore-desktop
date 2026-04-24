use crate::action::ConnectionSubPane;
use crate::app::App;
use crate::state::FocusTarget;
use crate::theme;
use crate::util::i18n::t;
use crate::util::unicode::truncate;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(0)])
        .split(area);

    render_sub_pane_menu(frame, chunks[0], app);
    render_body(frame, chunks[1], app);
}

fn render_sub_pane_menu(frame: &mut Frame, area: Rect, app: &App) {
    let items = [
        (ConnectionSubPane::BleScan, "BLE".to_string()),
        (ConnectionSubPane::SerialList, "Série".to_string()),
        (ConnectionSubPane::TcpInput, "TCP".to_string()),
        (ConnectionSubPane::Active, "Actives".to_string()),
        (
            ConnectionSubPane::Companions,
            t("conn.companions.label"),
        ),
    ];
    let mut lines: Vec<Line> = items
        .iter()
        .map(|(pane, label)| {
            let selected = std::mem::discriminant(&app.connection_ui.sub_pane)
                == std::mem::discriminant(pane);
            if selected {
                Line::from(Span::styled(
                    format!(" ▶ {} ", label),
                    theme::selected_row(),
                ))
            } else {
                Line::from(Span::raw(format!("   {} ", label)))
            }
        })
        .collect();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  {}", t("conn.arrow_hint")),
        theme::dim(),
    )));

    let focused = matches!(app.ui.focus, FocusTarget::ConnSubPane);
    let title = format!(" {} ", t("conn.sources"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    match app.connection_ui.sub_pane {
        ConnectionSubPane::BleScan => render_ble(frame, area, app),
        ConnectionSubPane::SerialList => render_serial(frame, area, app),
        ConnectionSubPane::TcpInput => render_tcp(frame, area, app),
        ConnectionSubPane::Active => render_active(frame, area, app),
        ConnectionSubPane::Companions => render_companions(frame, area, app),
    }
}

fn render_companions(frame: &mut Frame, area: Rect, app: &App) {
    let focused = matches!(app.ui.focus, FocusTarget::ConnList);
    let title = format!(" {} ", t("conn.companions.title"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    if app.connection_ui.companions.is_empty() {
        let body = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("conn.companions.empty")),
                theme::dim(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("conn.companions.hint_empty")),
                theme::dim(),
            )),
        ])
        .block(block);
        frame.render_widget(body, area);
        return;
    }

    let items: Vec<ListItem> = app
        .connection_ui
        .companions
        .iter()
        .map(|c| {
            let transport_icon = match c.transport_type.as_str() {
                "ble" => "📶",
                "serial" => "🔌",
                "tcp" => "🌐",
                _ => "·",
            };
            ListItem::new(Line::from(vec![
                Span::raw(format!(" {} ", transport_icon)),
                Span::raw(truncate(&c.name, 28)),
                Span::styled(format!("  {}", c.last_used), theme::dim()),
            ]))
        })
        .collect();

    let mut list_state = app.connection_ui.companions_list_state.clone();
    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selected_row())
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_ble(frame: &mut Frame, area: Rect, app: &App) {
    let title = if app.connection_ui.ble_scanning {
        format!(" {} ", t("conn.ble.scanning"))
    } else {
        format!(" {} ", t("conn.ble.idle"))
    };
    let focused = matches!(app.ui.focus, FocusTarget::ConnList);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    if app.connection_ui.ble_devices.is_empty() {
        let body = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("conn.ble.empty")),
                theme::dim(),
            )),
            Line::from(""),
            Line::from(Span::raw(format!("  {}", t("conn.ble.hint_scan")))),
        ])
        .block(block);
        frame.render_widget(body, area);
        return;
    }

    let items: Vec<ListItem> = app
        .connection_ui
        .ble_devices
        .iter()
        .map(|d| {
            let rssi = d
                .rssi
                .map(|r| format!(" {:>4} dBm", r))
                .unwrap_or_else(|| "       ".to_string());
            ListItem::new(Line::from(vec![
                Span::raw(truncate(&d.name, 26)),
                Span::styled(rssi, theme::dim()),
                Span::styled(format!("  {}", d.address), theme::dim()),
            ]))
        })
        .collect();

    let mut list_state = app.connection_ui.ble_list_state.clone();
    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selected_row())
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_serial(frame: &mut Frame, area: Rect, app: &App) {
    let focused = matches!(app.ui.focus, FocusTarget::ConnList);
    let title = if app.connection_ui.serial_scanning {
        format!(" {} ", t("conn.serial.scanning"))
    } else {
        format!(" {} ", t("conn.serial.idle"))
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    if app.connection_ui.serial_ports.is_empty() {
        let body = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("conn.serial.empty")),
                theme::dim(),
            )),
            Line::from(""),
            Line::from(Span::raw(format!("  {}", t("conn.serial.hint_scan")))),
        ])
        .block(block);
        frame.render_widget(body, area);
        return;
    }

    let items: Vec<ListItem> = app
        .connection_ui
        .serial_ports
        .iter()
        .map(|p| ListItem::new(Line::from(Span::raw(p.clone()))))
        .collect();

    let mut list_state = app.connection_ui.serial_list_state.clone();
    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selected_row())
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_tcp(frame: &mut Frame, area: Rect, app: &App) {
    let focused = matches!(app.ui.focus, FocusTarget::ConnTcpInput);
    let title = format!(" {} ", t("conn.tcp.title"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    let cursor = if focused { "_" } else { "" };
    let lines = vec![
        Line::from(""),
        Line::from(Span::raw(format!("  {}", t("conn.tcp.prompt")))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  > ", theme::title()),
            Span::raw(app.connection_ui.tcp_input.clone()),
            Span::styled(cursor, theme::dim()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", t("conn.tcp.hint")),
            theme::dim(),
        )),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn render_active(frame: &mut Frame, area: Rect, app: &App) {
    let focused = matches!(app.ui.focus, FocusTarget::ConnList);
    let title = format!(" {} ", t("conn.active.title"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    if app.connection_ui.active_connections.is_empty() {
        let body = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("conn.active.empty")),
                theme::dim(),
            )),
            Line::from(""),
            Line::from(Span::raw(format!("  {}", t("conn.active.hint_r")))),
            Line::from(Span::raw(format!("  {}", t("conn.active.hint_scan")))),
        ])
        .block(block);
        frame.render_widget(body, area);
        return;
    }

    let items: Vec<ListItem> = app
        .connection_ui
        .active_connections
        .iter()
        .map(|c| {
            let prefix = if c.is_primary { "★ " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, theme::ok_style()),
                Span::raw(c.label.clone()),
            ]))
        })
        .collect();

    let mut list_state = app.connection_ui.active_list_state.clone();
    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selected_row())
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut list_state);
}
