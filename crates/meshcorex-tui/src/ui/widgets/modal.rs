use crate::action::{ModalKind, Tab};
use crate::theme;
use crate::util::i18n::t;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

#[allow(clippy::too_many_arguments)]
pub fn render(
    frame: &mut Frame,
    area: Rect,
    modal: &ModalKind,
    current_tab: &Tab,
    tcp_input: &str,
    device_name_input: &str,
    tx_power_draft: u8,
    channel_edit_name: &str,
    channel_edit_notifications: bool,
    channel_edit_scope: &str,
    channel_edit_field: u8,
    channel_edit_psk_hex: &str,
    channel_new_name: &str,
    channel_new_psk_hex: &str,
    channel_new_field: u8,
    channel_new_idx: Option<u8>,
    room_login_password: &str,
) {
    // Cas spécial : ChannelEdit a besoin de ses propres champs
    if let ModalKind::ChannelEdit { idx } = modal {
        render_channel_edit(
            frame,
            area,
            *idx,
            channel_edit_name,
            channel_edit_notifications,
            channel_edit_scope,
            channel_edit_field,
            channel_edit_psk_hex,
        );
        return;
    }
    if matches!(modal, ModalKind::ChannelNew) {
        render_channel_new(
            frame,
            area,
            channel_new_name,
            channel_new_psk_hex,
            channel_new_field,
            channel_new_idx,
        );
        return;
    }
    let (title, body): (String, Vec<Line>) = match modal {
        ModalKind::ConfirmDeleteContact { name, .. } => (
            format!(" {} ", t("modal.confirm_delete.title")),
            vec![
                Line::from(""),
                Line::from(Span::raw(
                    t("modal.confirm_delete.contact").replace("{}", name),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[y]", theme::title()),
                    Span::raw(format!(" {}    ", t("common.yes"))),
                    Span::styled("[n]", theme::title()),
                    Span::raw(format!(" {}", t("common.no"))),
                ]),
            ],
        ),
        ModalKind::ConfirmDeleteChannel { idx, name } => (
            format!(" {} ", t("modal.confirm_delete.title")),
            vec![
                Line::from(""),
                Line::from(Span::raw(
                    t("modal.confirm_delete.channel")
                        .replacen("{}", &idx.to_string(), 1)
                        .replacen("{}", name, 1),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    format!("  {}", t("modal.confirm_delete.channel_note")),
                    theme::dim(),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[y]", theme::title()),
                    Span::raw(format!(" {}    ", t("common.yes"))),
                    Span::styled("[n]", theme::title()),
                    Span::raw(format!(" {}", t("common.no"))),
                ]),
            ],
        ),
        ModalKind::HelpOverlay => (format!(" {} ", t("help.title")), help_lines(current_tab)),
        ModalKind::TcpConnect => (
            format!(" {} ", t("modal.tcp.title")),
            vec![
                Line::from(""),
                Line::from(Span::raw(t("modal.tcp.prompt"))),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" > ", theme::title()),
                    Span::raw(tcp_input.to_string()),
                    Span::styled("_", theme::dim()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Enter]", theme::title()),
                    Span::raw(format!(" {}  ", t("common.connect"))),
                    Span::styled("[Esc]", theme::title()),
                    Span::raw(format!(" {}", t("common.cancel"))),
                ]),
            ],
        ),
        ModalKind::DeviceSetName => (
            format!(" {} ", t("modal.device_name.title")),
            vec![
                Line::from(""),
                Line::from(Span::raw(t("modal.device_name.prompt"))),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" > ", theme::title()),
                    Span::raw(device_name_input.to_string()),
                    Span::styled("_", theme::dim()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Enter]", theme::title()),
                    Span::raw(format!(" {}  ", t("common.confirm"))),
                    Span::styled("[Esc]", theme::title()),
                    Span::raw(format!(" {}", t("common.cancel"))),
                ]),
            ],
        ),
        ModalKind::DeviceSetTxPower => (
            format!(" {} ", t("modal.tx_power.title")),
            vec![
                Line::from(""),
                Line::from(Span::raw(t("modal.tx_power.hint"))),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" > ", theme::title()),
                    Span::raw(format!("{}", tx_power_draft)),
                    Span::raw(" dBm"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Enter]", theme::title()),
                    Span::raw(format!(" {}  ", t("modal.tx_power.send"))),
                    Span::styled("[Esc]", theme::title()),
                    Span::raw(format!(" {}", t("common.cancel"))),
                ]),
            ],
        ),
        ModalKind::PairBle { addr, name } => (
            format!(" {} ", t("modal.pair_ble.title")),
            vec![
                Line::from(""),
                Line::from(Span::raw(format!(
                    "  {}",
                    t("modal.pair_ble.to").replace("{}", name),
                ))),
                Line::from(Span::styled(
                    format!("  {}", addr),
                    theme::dim(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    format!("  {}", t("modal.pair_ble.info1")),
                    theme::dim(),
                )),
                Line::from(Span::styled(
                    format!("  {}", t("modal.pair_ble.info2")),
                    theme::dim(),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Enter]", theme::title()),
                    Span::raw(format!(" {}  ", t("modal.pair_ble.pair_btn"))),
                    Span::styled("[Esc]", theme::title()),
                    Span::raw(format!(" {}", t("common.cancel"))),
                ]),
            ],
        ),
        ModalKind::RoomLogin { name, .. } => (
            format!(" {} ", t("modal.room_login.title")),
            vec![
                Line::from(""),
                Line::from(Span::raw(format!(
                    "  {}",
                    t("modal.room_login.to").replace("{}", name),
                ))),
                Line::from(""),
                Line::from(Span::styled(
                    format!("  {}", t("modal.room_login.prompt")),
                    theme::dim(),
                )),
                Line::from(vec![
                    Span::styled(" > ", theme::title()),
                    Span::raw("•".repeat(room_login_password.chars().count())),
                    Span::styled("_", theme::dim()),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    format!("  {}", t("modal.room_login.info")),
                    theme::dim(),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Enter]", theme::title()),
                    Span::raw(format!(" {}  ", t("common.connect"))),
                    Span::styled("[Esc]", theme::title()),
                    Span::raw(format!(" {}", t("common.cancel"))),
                ]),
            ],
        ),
        ModalKind::ConfirmReboot => (
            format!(" {} ", t("modal.reboot.title")),
            vec![
                Line::from(""),
                Line::from(Span::raw(t("modal.reboot.prompt"))),
                Line::from(Span::styled(
                    t("modal.reboot.warning"),
                    theme::dim(),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[y]", theme::title()),
                    Span::raw(format!(" {}    ", t("common.yes"))),
                    Span::styled("[n]", theme::title()),
                    Span::raw(format!(" {}", t("common.no"))),
                ]),
            ],
        ),
        ModalKind::ChannelEdit { .. } | ModalKind::ChannelNew => {
            // Déjà géré en court-circuit en tête de fonction
            return;
        }
        ModalKind::ContactInfo { .. } | ModalKind::RepeaterAdmin { .. } => {
            // Rendu plein-écran dans ui::mod.rs (ne doit pas arriver ici)
            return;
        }
    };

    let rect = centered_rect(60, 40, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::focused_border())
        .title(title);

    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(body)
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false }),
        rect,
    );
}

fn help_lines(current_tab: &Tab) -> Vec<Line<'static>> {
    let kv = |key: &str, tr: &str| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("  {:<8}", key), theme::title()),
            Span::raw(tr.to_string()),
        ])
    };
    let mut lines = vec![
        Line::from(""),
        kv("F1-F6", &t("help.tabs")),
        kv("1-6", &t("help.tabs_alt")),
        kv("Alt+1-6", &t("help.tabs_alt2")),
        kv("Tab", &t("help.tab_focus")),
        kv("↑ ↓", &t("help.list_nav")),
        kv("Enter", &t("help.select")),
        kv("?", &t("help.toggle_help")),
        kv("q", &t("help.quit")),
        Line::from(""),
    ];
    // Section contextuelle : uniquement les raccourcis de la tab active
    match current_tab {
        Tab::Connection => {
            lines.push(Line::from(Span::styled(
                t("help.section.conn"),
                theme::title(),
            )));
            lines.push(kv("← →", &t("help.conn.panels")));
            lines.push(kv("s", &t("help.conn.scan")));
            lines.push(kv("r", &t("help.conn.refresh")));
            lines.push(kv("R", &t("help.conn.reconnect")));
            lines.push(kv("Enter", &t("help.conn.connect")));
            lines.push(kv("d", &t("help.conn.disconnect")));
            lines.push(kv("P", &t("help.conn.pair")));
        }
        Tab::Contacts => {
            lines.push(Line::from(Span::styled(
                t("help.section.contacts"),
                theme::title(),
            )));
            lines.push(kv("i", &t("help.contacts.info")));
            lines.push(kv("s", &t("help.contacts.sync")));
            lines.push(kv("f", &t("help.contacts.fav")));
            lines.push(kv("d", &t("help.contacts.delete")));
            lines.push(kv("t", &t("help.contacts.sort")));
            lines.push(kv("R", &t("help.contacts.repeater")));
        }
        Tab::Chat => {
            lines.push(Line::from(Span::styled(
                t("help.section.chat"),
                theme::title(),
            )));
            lines.push(kv("Tab", &t("help.chat.focus")));
            lines.push(kv("↑ ↓", &t("help.chat.nav")));
            lines.push(kv("Enter", &t("help.chat.enter")));
            lines.push(kv("PgUp", &t("help.chat.older")));
            lines.push(kv("@", &t("help.chat.mention")));
        }
        Tab::Channels => {
            lines.push(Line::from(Span::styled(
                t("help.section.channels"),
                theme::title(),
            )));
            lines.push(kv("n", &t("help.channels.new")));
            lines.push(kv("e", &t("help.channels.edit")));
            lines.push(kv("Enter", &t("help.channels.open")));
            lines.push(kv("r", &t("help.channels.read")));
            lines.push(kv("s", &t("help.channels.sync")));
            lines.push(kv("d", &t("help.channels.delete")));
        }
        Tab::Device => {
            lines.push(Line::from(Span::styled(
                t("help.section.device"),
                theme::title(),
            )));
            lines.push(kv("n", &t("help.device.name")));
            lines.push(kv("p", &t("help.device.tx_power")));
            lines.push(kv("t", &t("help.device.sync_time")));
            lines.push(kv("a", &t("help.device.advert")));
            lines.push(kv("A", &t("help.device.advert_flood")));
            lines.push(kv("b", &t("help.device.battery")));
            lines.push(kv("R", &t("help.device.refresh")));
            lines.push(kv("B", &t("help.device.reboot")));
            lines.push(kv("c", &t("help.device.chem")));
        }
        Tab::Settings => {
            lines.push(Line::from(Span::styled(
                t("help.section.settings"),
                theme::title(),
            )));
            lines.push(kv("← →", &t("help.settings.cycle")));
            lines.push(kv("Enter", &t("help.settings.cycle")));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(t("help.close_hint"), theme::dim())));
    lines
}

fn render_channel_new(
    frame: &mut Frame,
    area: Rect,
    name: &str,
    psk_hex: &str,
    active_field: u8,
    next_idx: Option<u8>,
) {
    let rect = centered_rect(70, 60, area);
    let idx_label = next_idx
        .map(|i| format!("#{}", i))
        .unwrap_or_else(|| t("channel.new.slot_none"));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::focused_border())
        .title(format!(
            " {} ",
            t("channel.new.title").replace("{}", &idx_label),
        ));

    let arrow = |field: u8| if active_field == field { "▶ " } else { "  " };
    let name_style = if active_field == 0 { theme::title() } else { theme::dim() };
    let psk_style = if active_field == 1 { theme::title() } else { theme::dim() };
    let cursor_if = |field: u8| if active_field == field { "_" } else { "" };

    let is_hashtag = name.trim().starts_with('#');
    let psk_status = match psk_hex.len() {
        0 if is_hashtag => t("channel.new.psk_empty_hashtag"),
        0 => t("channel.new.psk_empty"),
        32 => t("channel.new.psk_ok"),
        n => t("channel.new.psk_partial").replace("{}", &n.to_string()),
    };

    let mut body = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(arrow(0), theme::title()),
            Span::styled(t("channel.new.name_label"), name_style),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("> ", theme::title()),
            Span::raw(name.to_string()),
            Span::styled(cursor_if(0), theme::dim()),
        ]),
    ];
    if is_hashtag {
        body.push(Line::from(Span::styled(
            format!("    {}", t("channel.new.hashtag_info1")),
            theme::ok_style(),
        )));
        body.push(Line::from(Span::styled(
            format!("    {}", t("channel.new.hashtag_info2")),
            theme::dim(),
        )));
    }
    body.extend_from_slice(&[
        Line::from(""),
        Line::from(vec![
            Span::styled(arrow(1), theme::title()),
            Span::styled(
                t("channel.new.psk_label").replace("{}", &psk_status),
                psk_style,
            ),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("> ", theme::title()),
            Span::raw(psk_hex.to_string()),
            Span::styled(cursor_if(1), theme::dim()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", t("channel.new.aes_info")),
            theme::dim(),
        )),
        Line::from(Span::styled(
            format!("  {}", t("channel.new.private_info")),
            theme::dim(),
        )),
        Line::from(Span::styled(
            format!("  {}", t("channel.new.hashtag_info")),
            theme::dim(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[F3]", theme::title()),
            Span::raw(format!(" {}  ", t("channel.new.action_random"))),
            Span::styled("[F4]", theme::title()),
            Span::raw(format!(" {}  ", t("channel.new.action_derive"))),
            Span::styled("[Tab]", theme::title()),
            Span::raw(format!(" {}", t("channel.new.action_field"))),
        ]),
        Line::from(vec![
            Span::styled("[Enter]", theme::title()),
            Span::raw(format!(" {}  ", t("channel.new.action_submit"))),
            Span::styled("[Esc]", theme::title()),
            Span::raw(format!(" {}", t("common.cancel"))),
        ]),
    ]);

    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(body)
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false }),
        rect,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_channel_edit(
    frame: &mut Frame,
    area: Rect,
    idx: u8,
    name: &str,
    notifications: bool,
    scope: &str,
    active_field: u8,
    psk_hex: &str,
) {
    let rect = centered_rect(72, 70, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::focused_border())
        .title(format!(
            " {} ",
            t("channel.edit.title").replace("{}", &idx.to_string()),
        ));

    let checkbox = if notifications { "[x]" } else { "[ ]" };
    let notif_label = if notifications {
        t("channel.edit.notif_on")
    } else {
        t("channel.edit.notif_off")
    };
    let cursor_if = |field: u8| if active_field == field { "_" } else { "" };
    let arrow = |field: u8| if active_field == field { "▶ " } else { "  " };

    let name_style = if active_field == 0 { theme::title() } else { theme::dim() };
    let notif_style = if active_field == 1 { theme::title() } else { theme::dim() };
    let scope_style = if active_field == 2 { theme::title() } else { theme::dim() };

    let body = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(arrow(0), theme::title()),
            Span::styled(t("channel.edit.name_label"), name_style),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("> ", theme::title()),
            Span::raw(name.to_string()),
            Span::styled(cursor_if(0), theme::dim()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(arrow(1), theme::title()),
            Span::styled(
                format!("{}  {}", checkbox, notif_label),
                notif_style,
            ),
            Span::styled(
                if active_field == 1 { t("channel.edit.notif_hint") } else { String::new() },
                theme::dim(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(arrow(2), theme::title()),
            Span::styled(t("channel.edit.scope_label"), scope_style),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("> ", theme::title()),
            Span::raw(scope.to_string()),
            Span::styled(cursor_if(2), theme::dim()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", t("channel.edit.scope_info")),
            theme::dim(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  {}", t("channel.edit.psk_label")), theme::title()),
            Span::styled(psk_hex.to_string(), theme::dim()),
        ]),
        Line::from(Span::styled(
            format!("  {}", t("channel.edit.psk_copy_hint")),
            theme::dim(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Tab]", theme::title()),
            Span::raw(" / "),
            Span::styled("[Shift-Tab]", theme::title()),
            Span::raw(format!(" {}  ", t("channel.edit.action_field"))),
            Span::styled("[Espace]", theme::title()),
            Span::raw(format!(" {}", t("channel.edit.action_toggle"))),
        ]),
        Line::from(vec![
            Span::styled("[Enter]", theme::title()),
            Span::raw(format!(" {}  ", t("channel.edit.action_save"))),
            Span::styled("[F2]", theme::title()),
            Span::raw(format!(" {}  ", t("channel.edit.action_sync"))),
            Span::styled("[F5]", theme::title()),
            Span::raw(format!(" {}  ", t("channel.edit.action_copy_psk"))),
            Span::styled("[Esc]", theme::title()),
            Span::raw(format!(" {}", t("common.cancel"))),
        ]),
    ];

    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(body)
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false }),
        rect,
    );
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
