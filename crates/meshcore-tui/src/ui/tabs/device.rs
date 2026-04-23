use crate::app::App;
use crate::theme;
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
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Informations device ")
        .border_style(theme::unfocused_border());

    let lines: Vec<Line> = if let Some(info) = &app.device_ui.info {
        vec![
            info_line("Nom", &info.name),
            info_line("Pubkey", &info.public_key),
            info_line(
                "Radio",
                &format!(
                    "{:.3} MHz · BW {:.1} kHz · SF{} · CR4/{}",
                    info.radio_freq as f32 / 1000.0,
                    info.radio_bw as f32 / 1000.0,
                    info.sf,
                    info.cr + 4
                ),
            ),
            info_line(
                "TX power",
                &format!("{} dBm (max {} dBm)", info.tx_power, info.max_tx_power),
            ),
            info_line(
                "GPS",
                &format!("{:.5}, {:.5}", info.lat, info.lon),
            ),
            info_line("Type advert", &format!("{}", info.adv_type)),
        ]
    } else if app.ui.connected {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Chargement des infos device…",
                theme::dim(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Appuyer sur [R] pour rafraîchir",
                theme::dim(),
            )),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Non connecté — utilisez la tab 1 pour vous connecter.",
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
    let title = format!(" Batterie · chimie : {} · [c] changer ", chem);
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
        None => String::from("— · [b] lire"),
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
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Actions ")
        .border_style(theme::unfocused_border());

    let conn = app.ui.connected;
    let dim = theme::dim();
    let title_style = theme::title();

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [n]", title_style),
        Span::styled(
            if conn { " Changer le nom" } else { " (déconnecté) Changer le nom" },
            if conn { Style::default() } else { dim },
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [p]", title_style),
        Span::raw(" Ajuster TX power (+/- pour régler, Entrée pour valider)"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [t]", title_style),
        Span::raw(" Synchroniser l'heure"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [a]", title_style),
        Span::raw(" Envoyer un advert (normal)     "),
        Span::styled("[A]", title_style),
        Span::raw(" flood (pour le réseau)"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [b]", title_style),
        Span::raw(" Relire la batterie"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [R]", title_style),
        Span::raw(" Rafraîchir toutes les infos"),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [B]", theme::err_style()),
        Span::raw(" Redémarrer le device (avec confirmation)"),
    ]));

    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), area);
}

fn render_hints(frame: &mut Frame, area: Rect) {
    let spans = vec![
        Span::styled("  ", theme::dim()),
        Span::styled("Tab 2", theme::title()),
        Span::styled(" puis ", theme::dim()),
        Span::styled("R", theme::title()),
        Span::styled(" sur un repeater → administration distante  ·  ", theme::dim()),
        Span::styled("?", theme::title()),
        Span::styled(" aide", theme::dim()),
    ];
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
