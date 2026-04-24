use crate::action::ModalKind;
use crate::theme;
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
    let (title, body): (&str, Vec<Line>) = match modal {
        ModalKind::ConfirmDeleteContact { name, .. } => (
            " Confirmer suppression ",
            vec![
                Line::from(""),
                Line::from(Span::raw(format!(
                    "Supprimer le contact « {} » ?",
                    name
                ))),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[y]", theme::title()),
                    Span::raw(" Oui    "),
                    Span::styled("[n]", theme::title()),
                    Span::raw(" Non"),
                ]),
            ],
        ),
        ModalKind::ConfirmDeleteChannel { idx, name } => (
            " Confirmer suppression ",
            vec![
                Line::from(""),
                Line::from(Span::raw(format!(
                    "Supprimer le canal #{} « {} » ?",
                    idx, name
                ))),
                Line::from(""),
                Line::from(Span::styled(
                    "  (le canal sera aussi effacé sur le device)",
                    theme::dim(),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[y]", theme::title()),
                    Span::raw(" Oui    "),
                    Span::styled("[n]", theme::title()),
                    Span::raw(" Non"),
                ]),
            ],
        ),
        ModalKind::HelpOverlay => (" Raccourcis ", help_lines()),
        ModalKind::TcpConnect => (
            " Connexion TCP ",
            vec![
                Line::from(""),
                Line::from(Span::raw("Saisir host:port (ex: 192.168.1.50:4403)")),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" > ", theme::title()),
                    Span::raw(tcp_input.to_string()),
                    Span::styled("_", theme::dim()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Enter]", theme::title()),
                    Span::raw(" Connecter  "),
                    Span::styled("[Esc]", theme::title()),
                    Span::raw(" Annuler"),
                ]),
            ],
        ),
        ModalKind::DeviceSetName => (
            " Renommer le device ",
            vec![
                Line::from(""),
                Line::from(Span::raw("Nouveau nom :")),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" > ", theme::title()),
                    Span::raw(device_name_input.to_string()),
                    Span::styled("_", theme::dim()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Enter]", theme::title()),
                    Span::raw(" Valider  "),
                    Span::styled("[Esc]", theme::title()),
                    Span::raw(" Annuler"),
                ]),
            ],
        ),
        ModalKind::DeviceSetTxPower => (
            " Ajuster TX power ",
            vec![
                Line::from(""),
                Line::from(Span::raw("Utiliser + / - ou ↑ / ↓ pour régler, Entrée pour valider.")),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" > ", theme::title()),
                    Span::raw(format!("{}", tx_power_draft)),
                    Span::raw(" dBm"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Enter]", theme::title()),
                    Span::raw(" Envoyer au device  "),
                    Span::styled("[Esc]", theme::title()),
                    Span::raw(" Annuler"),
                ]),
            ],
        ),
        ModalKind::ConfirmReboot => (
            " Confirmer le redémarrage ",
            vec![
                Line::from(""),
                Line::from(Span::raw("Redémarrer le device ?")),
                Line::from(Span::styled(
                    "(la connexion va être perdue quelques secondes)",
                    theme::dim(),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[y]", theme::title()),
                    Span::raw(" Oui    "),
                    Span::styled("[n]", theme::title()),
                    Span::raw(" Non"),
                ]),
            ],
        ),
        ModalKind::ChannelEdit { .. } | ModalKind::ChannelNew => {
            // Déjà géré en court-circuit en tête de fonction
            return;
        }
        ModalKind::RepeaterAdmin { .. } => {
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

fn help_lines() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  F1-F5   ", theme::title()),
            Span::raw("Naviguer entre les onglets"),
        ]),
        Line::from(vec![
            Span::styled("  1-5     ", theme::title()),
            Span::raw("Alternatif (si F1-F5 capté par le terminal)"),
        ]),
        Line::from(vec![
            Span::styled("  Alt+1-5 ", theme::title()),
            Span::raw("Alternatif, actif même pendant la saisie TCP"),
        ]),
        Line::from(vec![
            Span::styled("  Tab     ", theme::title()),
            Span::raw("Changer de focus dans l'onglet"),
        ]),
        Line::from(vec![
            Span::styled("  ↑ ↓    ", theme::title()),
            Span::raw("Naviguer dans une liste"),
        ]),
        Line::from(vec![
            Span::styled("  Enter   ", theme::title()),
            Span::raw("Sélectionner / Valider"),
        ]),
        Line::from(vec![
            Span::styled("  ?       ", theme::title()),
            Span::raw("Afficher/masquer cette aide"),
        ]),
        Line::from(vec![
            Span::styled("  q       ", theme::title()),
            Span::raw("Quitter"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Connexion (tab 1)", theme::title())),
        Line::from(vec![
            Span::styled("  ← →     ", theme::title()),
            Span::raw("Naviguer entre sous-panneaux (BLE/Série/TCP/Actives)"),
        ]),
        Line::from(vec![
            Span::styled("  s       ", theme::title()),
            Span::raw("Scanner le panneau actif (BLE ou Série)"),
        ]),
        Line::from(vec![
            Span::styled("  r       ", theme::title()),
            Span::raw("Rafraîchir la liste des connexions"),
        ]),
        Line::from(vec![
            Span::styled("  R       ", theme::title()),
            Span::raw("Reconnecter au dernier device connu"),
        ]),
        Line::from(vec![
            Span::styled("  Enter   ", theme::title()),
            Span::raw("Connecter à l'élément sélectionné"),
        ]),
        Line::from(vec![
            Span::styled("  d       ", theme::title()),
            Span::raw("Déconnecter"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Chat (tab 3)", theme::title())),
        Line::from(vec![
            Span::styled("  Tab     ", theme::title()),
            Span::raw("Cycle focus (liste / historique / saisie)"),
        ]),
        Line::from(vec![
            Span::styled("  ↑ ↓    ", theme::title()),
            Span::raw("Naviguer dans la zone focus"),
        ]),
        Line::from(vec![
            Span::styled("  Enter   ", theme::title()),
            Span::raw("Ouvrir la conversation / Envoyer le message"),
        ]),
        Line::from(vec![
            Span::styled("  PgUp    ", theme::title()),
            Span::raw("Charger les messages plus anciens"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Canaux (tab 4)", theme::title())),
        Line::from(vec![
            Span::styled("  Enter   ", theme::title()),
            Span::raw("Ouvrir dans le Chat"),
        ]),
        Line::from(vec![
            Span::styled("  r       ", theme::title()),
            Span::raw("Marquer comme lu"),
        ]),
        Line::from(vec![
            Span::styled("  s       ", theme::title()),
            Span::raw("Synchroniser sur le device"),
        ]),
        Line::from(vec![
            Span::styled("  d       ", theme::title()),
            Span::raw("Supprimer (avec confirmation)"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Contacts (tab 2)", theme::title())),
        Line::from(vec![
            Span::styled("  s       ", theme::title()),
            Span::raw("Synchroniser depuis le device"),
        ]),
        Line::from(vec![
            Span::styled("  f       ", theme::title()),
            Span::raw("Basculer favori"),
        ]),
        Line::from(vec![
            Span::styled("  d       ", theme::title()),
            Span::raw("Supprimer (avec confirmation)"),
        ]),
        Line::from(vec![
            Span::styled("  t       ", theme::title()),
            Span::raw("Basculer le tri (favoris / type / nom)"),
        ]),
        Line::from(""),
        Line::from(Span::styled("[Esc] pour fermer", theme::dim())),
    ]
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
        .unwrap_or_else(|| "aucun libre".to_string());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::focused_border())
        .title(format!(" Nouveau canal — slot {} ", idx_label));

    let arrow = |field: u8| if active_field == field { "▶ " } else { "  " };
    let name_style = if active_field == 0 { theme::title() } else { theme::dim() };
    let psk_style = if active_field == 1 { theme::title() } else { theme::dim() };
    let cursor_if = |field: u8| if active_field == field { "_" } else { "" };

    let is_hashtag = name.trim().starts_with('#');
    let psk_status = match psk_hex.len() {
        0 if is_hashtag => "(vide — dérivé auto depuis le nom au submit)".to_string(),
        0 => "(vide — [F3] aléatoire  [F4] dériver du nom)".to_string(),
        32 => "(32 hex OK)".to_string(),
        n => format!("({}/32 hex)", n),
    };

    let mut body = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(arrow(0), theme::title()),
            Span::styled("Nom du canal :", name_style),
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
            "    ↳ hashtag room : PSK = SHA256(nom)[:16] — tout le monde",
            theme::ok_style(),
        )));
        body.push(Line::from(Span::styled(
            "      qui tape ce nom obtient la même clé automatiquement",
            theme::dim(),
        )));
    }
    body.extend_from_slice(&[
        Line::from(""),
        Line::from(vec![
            Span::styled(arrow(1), theme::title()),
            Span::styled(
                format!("PSK hex — 32 caractères = 16 octets {}", psk_status),
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
            "  La clé partagée est utilisée pour le chiffrement AES-128 du canal.",
            theme::dim(),
        )),
        Line::from(Span::styled(
            "  Canal privé : tous les membres doivent avoir la même clé (QR code, copier).",
            theme::dim(),
        )),
        Line::from(Span::styled(
            "  Hashtag room (nom commençant par #) : clé dérivée automatiquement.",
            theme::dim(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[F3]", theme::title()),
            Span::raw(" PSK aléatoire  "),
            Span::styled("[F4]", theme::title()),
            Span::raw(" Dériver du nom (hashtag)  "),
            Span::styled("[Tab]", theme::title()),
            Span::raw(" champ"),
        ]),
        Line::from(vec![
            Span::styled("[Enter]", theme::title()),
            Span::raw(" Créer + sync device  "),
            Span::styled("[Esc]", theme::title()),
            Span::raw(" Annuler"),
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
        .title(format!(" Éditer le canal #{} ", idx));

    let checkbox = if notifications { "[x]" } else { "[ ]" };
    let notif_label = if notifications {
        "Notifications activées 🔔"
    } else {
        "Notifications désactivées 🔕"
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
            Span::styled("Nom du canal :", name_style),
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
                if active_field == 1 { "   (Espace pour cocher/décocher)" } else { "" },
                theme::dim(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(arrow(2), theme::title()),
            Span::styled(
                "Scope / région (ex: us-ca, #Morbihan — vide = scope par défaut du nœud) :",
                scope_style,
            ),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("> ", theme::title()),
            Span::raw(scope.to_string()),
            Span::styled(cursor_if(2), theme::dim()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Le scope est stocké localement (comme l'app Android : côté client).",
            theme::dim(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  PSK (read-only) : ", theme::title()),
            Span::styled(psk_hex.to_string(), theme::dim()),
        ]),
        Line::from(Span::styled(
            "  [F5] copier le PSK dans le presse-papier (pour partager à un membre).",
            theme::dim(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Tab]", theme::title()),
            Span::raw(" / "),
            Span::styled("[Shift-Tab]", theme::title()),
            Span::raw(" champ  "),
            Span::styled("[Espace]", theme::title()),
            Span::raw(" cocher notifications"),
        ]),
        Line::from(vec![
            Span::styled("[Enter]", theme::title()),
            Span::raw(" Enregistrer  "),
            Span::styled("[F2]", theme::title()),
            Span::raw(" + sync device  "),
            Span::styled("[F5]", theme::title()),
            Span::raw(" copier PSK  "),
            Span::styled("[Esc]", theme::title()),
            Span::raw(" Annuler"),
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
