use crate::app::App;
use crate::state::repeater::{RepeaterPane, RepeaterUiState};
use crate::theme;
use meshcorex_service::repeater::RepeaterStatus;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
};

pub fn render(frame: &mut Frame, area: Rect, app: &App, pubkey: &str, name: &str) {
    frame.render_widget(Clear, area);

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::focused_border())
        .title(format!(" Administration repeater · {} ", name));
    frame.render_widget(outer, area);

    let inner = area.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 1,
    });

    // Si pas encore loggé : afficher l'écran password
    if !app.repeater_ui.logged_in {
        render_password(frame, inner, &app.repeater_ui, name);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    render_tabs_bar(frame, chunks[0], app);
    match app.repeater_ui.pane {
        RepeaterPane::Status => render_status(frame, chunks[1], &app.repeater_ui),
        RepeaterPane::Neighbours => render_neighbours(frame, chunks[1], &app.repeater_ui),
        RepeaterPane::Acl => render_acl(frame, chunks[1], &app.repeater_ui),
        RepeaterPane::Cli => render_cli(frame, chunks[1], &app.repeater_ui),
    }
    render_hints(frame, chunks[2], &app.repeater_ui, pubkey);
}

fn render_password(frame: &mut Frame, area: Rect, ui: &RepeaterUiState, name: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let info = Paragraph::new(vec![
        Line::from(Span::raw(format!("  Authentification requise sur « {} »", name))),
        Line::from(Span::styled(
            "  Saisir le mot de passe puis Entrée (Esc pour fermer)",
            theme::dim(),
        )),
    ]);
    frame.render_widget(info, chunks[0]);

    let value = ui.password_input.value();
    let masked = "•".repeat(value.chars().count());
    let cursor = if ui.password_mode { "_" } else { "" };
    let pw_block = Block::default()
        .borders(Borders::ALL)
        .title(" Mot de passe ")
        .border_style(theme::focused_border());
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::raw(masked),
            Span::styled(cursor, theme::dim()),
        ]))
        .block(pw_block),
        chunks[1],
    );

    if let Some(msg) = &ui.login_message {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(msg.clone(), theme::warn_style()))),
            chunks[2],
        );
    }
}

fn render_tabs_bar(frame: &mut Frame, area: Rect, app: &App) {
    let titles: Vec<Line> = RepeaterPane::all()
        .iter()
        .map(|p| Line::from(Span::raw(p.label())))
        .collect();
    let selected = RepeaterPane::all()
        .iter()
        .position(|p| *p == app.repeater_ui.pane)
        .unwrap_or(0);
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::unfocused_border())
                .title(" Tab pour changer · Esc fermer "),
        )
        .select(selected)
        .highlight_style(theme::focused_border())
        .divider("│");
    frame.render_widget(tabs, area);
}

fn render_status(frame: &mut Frame, area: Rect, ui: &RepeaterUiState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Statut ")
        .border_style(theme::unfocused_border());

    let lines: Vec<Line> = match (&ui.status, &ui.status_error) {
        (Some(s), _) => status_lines(s),
        (None, _) if ui.loading => vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Récupération du statut en cours…",
                theme::dim(),
            )),
        ],
        (None, Some(err)) => error_lines(&crate::util::i18n::t("repeater.error.status"), err),
        (None, None) => vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Appuyer sur [r] pour charger le statut",
                theme::dim(),
            )),
        ],
    };
    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), area);
}

fn error_lines(title: &str, err: &str) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(Span::styled(format!("  ✗ {}", title), theme::err_style())),
        Line::from(""),
        Line::from(Span::raw(format!("  {}", err))),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("[r]", theme::title()),
            Span::raw(" réessayer  "),
            Span::styled("[c]", theme::title()),
            Span::raw(" fallback via CLI texte (panneau CLI)"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Les firmwares récents (zone géographique) ne répondent plus toujours",
            theme::dim(),
        )),
        Line::from(Span::styled(
            "  aux commandes binaires request_status / request_acl. Utilisez le CLI.",
            theme::dim(),
        )),
    ]
}

fn status_lines(s: &RepeaterStatus) -> Vec<Line<'static>> {
    use crate::util::i18n::t;
    vec![
        line_kv(&t("repeater.status.battery"), &format!("{} mV", s.battery_mv)),
        line_kv("TX queue", &format!("{}", s.tx_queue_len)),
        line_kv(&t("repeater.status.noise"), &format!("{} dBm", s.noise_floor)),
        line_kv(&t("repeater.status.rssi"), &format!("{} dBm · SNR {:.1}", s.last_rssi, s.snr)),
        line_kv(
            &t("repeater.status.traffic"),
            &format!(
                "{} recv / {} sent (flood {}, direct {})",
                s.nb_recv, s.nb_sent, s.flood_sent, s.direct_sent
            ),
        ),
        line_kv(
            &t("repeater.status.airtime"),
            &format!("TX {} / RX {}", s.airtime, s.rx_airtime),
        ),
        line_kv(&t("repeater.status.uptime"), &format!("{} s", s.uptime)),
        line_kv(&t("repeater.status.duplicates"), &format!("{}", s.dup_count)),
    ]
}

fn line_kv(k: &str, v: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {:<14} ", k), theme::title()),
        Span::raw(v.to_string()),
    ])
}

fn render_neighbours(frame: &mut Frame, area: Rect, ui: &RepeaterUiState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Voisins ({}) ", ui.neighbours.len()))
        .border_style(theme::unfocused_border());

    if ui.neighbours.is_empty() {
        if let Some(err) = &ui.neighbours_error {
            frame.render_widget(
                Paragraph::new(error_lines(&crate::util::i18n::t("repeater.error.neighbours"), err))
                    .block(block)
                    .wrap(Wrap { trim: false }),
                area,
            );
            return;
        }
        let msg = if ui.loading {
            "  Chargement des voisins…"
        } else {
            "  Appuyer sur [r] pour charger les voisins"
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(msg.to_string(), theme::dim())))
                .block(block),
            area,
        );
        return;
    }
    let items: Vec<ListItem> = ui
        .neighbours
        .iter()
        .map(|n| {
            let name = n.name.clone().unwrap_or_else(|| n.pubkey_hex[..8].to_string());
            ListItem::new(Line::from(vec![
                Span::raw(format!(" {:<22} ", name)),
                Span::styled(format!("SNR {:.1}  ", n.snr), theme::dim()),
                Span::styled(format!("{}s ago  ", n.secs_ago), theme::dim()),
                Span::styled(&n.pubkey_hex[..8.min(n.pubkey_hex.len())], theme::dim()),
            ]))
        })
        .collect();
    frame.render_widget(List::new(items).block(block), area);
}

fn render_acl(frame: &mut Frame, area: Rect, ui: &RepeaterUiState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" ACL ({}) ", ui.acl.len()))
        .border_style(theme::unfocused_border());

    if ui.acl.is_empty() {
        if let Some(err) = &ui.acl_error {
            frame.render_widget(
                Paragraph::new(error_lines(&crate::util::i18n::t("repeater.error.acl"), err))
                    .block(block)
                    .wrap(Wrap { trim: false }),
                area,
            );
            return;
        }
        let msg = if ui.loading {
            "  Chargement ACL…"
        } else {
            "  Appuyer sur [r] pour charger les règles ACL"
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(msg.to_string(), theme::dim())))
                .block(block),
            area,
        );
        return;
    }
    let items: Vec<ListItem> = ui
        .acl
        .iter()
        .map(|e| {
            let name = e.name.clone().unwrap_or_else(|| "?".to_string());
            ListItem::new(Line::from(vec![
                Span::raw(format!(" {:<22} ", name)),
                Span::styled(format!("perms=0x{:02x}  ", e.permissions), theme::dim()),
                Span::styled(&e.pubkey_hex[..8.min(e.pubkey_hex.len())], theme::dim()),
            ]))
        })
        .collect();
    frame.render_widget(List::new(items).block(block), area);
}

fn render_cli(frame: &mut Frame, area: Rect, ui: &RepeaterUiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    let out_block = Block::default()
        .borders(Borders::ALL)
        .title(" Sortie CLI ")
        .border_style(theme::unfocused_border());

    let lines: Vec<Line> = if ui.cli_output.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Tapez une commande puis Entrée pour l'envoyer au repeater.",
                theme::dim(),
            )),
            Line::from(Span::styled(
                "  Tapez `help` (ou `?`) pour la liste complète des commandes.",
                theme::dim(),
            )),
            Line::from(Span::styled(
                "  Exemples : ver · get name · get radio · clock · neighbors · advert",
                theme::dim(),
            )),
        ]
    } else {
        ui.cli_output
            .iter()
            .map(|l| Line::from(Span::raw(l.clone())))
            .collect()
    };
    let para = Paragraph::new(lines)
        .block(out_block)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left);
    frame.render_widget(para, chunks[0]);

    let in_block = Block::default()
        .borders(Borders::ALL)
        .title(" Commande (Entrée pour envoyer) ")
        .border_style(theme::focused_border());
    let value = ui.cli_input.value();
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" > "),
            Span::raw(value.to_string()),
            Span::styled("_", theme::dim()),
        ]))
        .block(in_block),
        chunks[1],
    );
}

fn render_hints(frame: &mut Frame, area: Rect, ui: &RepeaterUiState, _pubkey: &str) {
    let loading = if ui.loading { " · chargement…" } else { "" };
    let spans = vec![
        Span::styled("  ", theme::dim()),
        Span::styled("Tab", theme::title()),
        Span::styled(" panneau  ·  ", theme::dim()),
        Span::styled("r", theme::title()),
        Span::styled(" rafraîchir  ·  ", theme::dim()),
        Span::styled("L", theme::title()),
        Span::styled(" déconnexion  ·  ", theme::dim()),
        Span::styled("Esc", theme::title()),
        Span::styled(format!(" fermer{}", loading), theme::dim()),
    ];
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
