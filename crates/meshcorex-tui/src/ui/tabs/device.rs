use crate::app::App;
use crate::theme;
use crate::util::i18n::t;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11),
            Constraint::Length(5),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    render_info(frame, chunks[0], app);
    render_battery(frame, chunks[1], app);
    render_actions(frame, chunks[2], app);
    render_hints(frame, chunks[3]);
}

fn render_info(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!(" {} ", t("device.info.title"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(theme::unfocused_border());

    let lines: Vec<Line> = if let Some(info) = &app.device_ui.info {
        vec![
            info_line(&t("device.info.name"), &info.name),
            info_line(&t("device.info.pubkey"), &info.public_key),
            info_line(
                &t("device.info.radio"),
                &format!(
                    "{:.3} MHz · BW {:.1} kHz · SF{} · CR4/{}",
                    info.radio_freq as f32 / 1000.0,
                    info.radio_bw as f32 / 1000.0,
                    info.sf,
                    info.cr + 4
                ),
            ),
            info_line(
                &t("device.info.tx_power"),
                &format!("{} dBm (max {} dBm)", info.tx_power, info.max_tx_power),
            ),
            info_line(
                &t("device.info.gps"),
                &format!("{:.5}, {:.5}", info.lat, info.lon),
            ),
            info_line(&t("device.info.adv_type"), &format!("{}", info.adv_type)),
        ]
    } else if app.ui.connected {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("device.info.loading")),
                theme::dim(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("device.info.refresh_hint")),
                theme::dim(),
            )),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("device.info.not_connected")),
                theme::dim(),
            )),
        ]
    };
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn info_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {:<12} ", label), theme::title()),
        Span::raw(value.to_string()),
    ])
}

fn render_battery(frame: &mut Frame, area: Rect, app: &App) {
    let chem = app.device_ui.chemistry_label();
    let title = format!(
        " {} · {} : {} · {} ",
        t("device.battery.title"),
        t("device.battery.chem"),
        chem,
        t("device.battery.change"),
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(theme::unfocused_border());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let percent = app.device_ui.battery_percent.unwrap_or(0);
    let mv = app.device_ui.battery_mv;
    let label = match mv {
        Some(v) => format!("{} % ({} mV)", percent, v),
        None => t("device.battery.read_hint"),
    };
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(match percent {
            0..=19 => theme::ERR,
            20..=49 => theme::WARN,
            _ => theme::OK,
        }))
        .percent(percent as u16)
        .label(label);
    frame.render_widget(gauge, inner);
}

fn render_actions(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!(" {} ", t("device.actions.title"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(theme::unfocused_border());

    let conn = app.ui.connected;
    let dim = theme::dim();
    let title_style = theme::title();

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [n]", title_style),
        Span::styled(
            if conn {
                format!(" {}", t("device.action.name"))
            } else {
                format!(" {}", t("device.action.name_disconnected"))
            },
            if conn { Style::default() } else { dim },
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [p]", title_style),
        Span::raw(format!(" {}", t("device.action.tx_power"))),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [t]", title_style),
        Span::raw(format!(" {}", t("device.action.sync_time"))),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [a]", title_style),
        Span::raw(format!(" {}", t("device.action.advert"))),
        Span::styled("[A]", title_style),
        Span::raw(format!(" {}", t("device.action.flood"))),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [b]", title_style),
        Span::raw(format!(" {}", t("device.action.battery"))),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [R]", title_style),
        Span::raw(format!(" {}", t("device.action.refresh"))),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [B]", theme::err_style()),
        Span::raw(format!(" {}", t("device.action.reboot"))),
    ]));

    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), area);
}

fn render_hints(frame: &mut Frame, area: Rect) {
    let spans = vec![
        Span::styled("  ", theme::dim()),
        Span::styled("Tab 2", theme::title()),
        Span::styled(format!(" {} ", t("device.tab_hint")), theme::dim()),
        Span::styled("R", theme::title()),
        Span::styled(format!(" {}  ·  ", t("device.tab_hint_suffix")), theme::dim()),
        Span::styled("?", theme::title()),
        Span::styled(format!(" {}", t("statusbar.help")), theme::dim()),
    ];
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
