//! Exécution des commandes (one-shot et REPL)

use crate::ContactsAction;
use crate::SubCommand;
use crate::display;
use meshcore_protocol::types::BatteryChemistry;
use meshcore_service::AppState;
use meshcore_transport::manager::ConnectionTarget;

/// Exécute une sous-commande clap (mode one-shot)
pub async fn execute_subcommand(
    state: &AppState,
    cmd: SubCommand,
    json: bool,
) -> Result<(), String> {
    match cmd {
        SubCommand::Contacts { action } => match action.unwrap_or(ContactsAction::List) {
            ContactsAction::Sync => {
                let count = meshcore_service::contacts::sync_contacts(state).await?;
                if json {
                    println!("{{\"synced\":{}}}", count);
                } else {
                    println!("{} contacts synchronisés", count);
                }
            }
            ContactsAction::List => {
                let contacts = meshcore_service::contacts::get_all_contacts(state)?;
                display::contacts(&contacts, json);
            }
        },
        SubCommand::Send { dest, message } => {
            let text = message.join(" ");
            let pubkey = resolve_contact(state, &dest)?;
            let id =
                meshcore_service::messaging::send_direct_message(state, &pubkey, &text).await?;
            if json {
                println!("{{\"sent\":\"{}\"}}", id);
            } else {
                println!("Message envoyé (id: {})", &id[..8]);
            }
        }
        SubCommand::Channel { idx, message } => {
            let text = message.join(" ");
            let id = meshcore_service::messaging::send_channel_message(state, idx, &text).await?;
            if json {
                println!("{{\"sent\":\"{}\"}}", id);
            } else {
                println!("Message canal #{} envoyé (id: {})", idx, &id[..8]);
            }
        }
        SubCommand::Channels => {
            let channels = meshcore_service::channels::get_all_channels(state)?;
            display::channels(&channels, json);
        }
        SubCommand::Device => {
            let info = meshcore_service::device::get_device_info(state).await?;
            display::device_info(&info, json);
        }
        SubCommand::Battery { chemistry } => {
            let chem = match chemistry.as_str() {
                "lifepo4" => BatteryChemistry::LiFePO4,
                "nimh" => BatteryChemistry::NiMH,
                _ => BatteryChemistry::LiPo,
            };
            let (mv, pct) = meshcore_service::device::get_battery(state, chem).await?;
            display::battery(mv, pct, json);
        }
    }
    Ok(())
}

/// Exécute une commande REPL (ligne parsée en tokens)
pub async fn execute_repl(state: &AppState, tokens: &[String], json: bool) -> Result<bool, String> {
    if tokens.is_empty() {
        return Ok(false);
    }

    match tokens[0].as_str() {
        "quit" | "exit" | "q" => return Ok(true),

        "help" | "?" => {
            display::help();
        }

        "connect" => {
            if tokens.len() < 3 {
                return Err("Usage: connect serial|tcp|ble <args>".to_string());
            }
            let target = match tokens[1].as_str() {
                "serial" => {
                    let baud = tokens.get(3).and_then(|b| b.parse().ok()).unwrap_or(115200);
                    ConnectionTarget::Serial {
                        port: tokens[2].clone(),
                        baud_rate: baud,
                    }
                }
                "tcp" => {
                    let port = tokens.get(3).and_then(|p| p.parse().ok()).unwrap_or(4403);
                    ConnectionTarget::Tcp {
                        host: tokens[2].clone(),
                        port,
                    }
                }
                "ble" => ConnectionTarget::Ble {
                    name_or_addr: tokens[2].clone(),
                },
                _ => return Err("Usage: connect serial|tcp|ble <args>".to_string()),
            };
            meshcore_service::connection::connect(state, target).await?;
            println!("Connecté");
        }

        "disconnect" => {
            meshcore_service::connection::disconnect(state).await?;
            println!("Déconnecté");
        }

        "status" => {
            let connected = meshcore_service::connection::is_connected(state).await;
            if json {
                println!("{{\"connected\":{}}}", connected);
            } else {
                println!(
                    "Statut : {}",
                    if connected {
                        "connecté"
                    } else {
                        "déconnecté"
                    }
                );
            }
        }

        "contacts" => {
            if tokens.get(1).map(|s| s.as_str()) == Some("sync") {
                let count = meshcore_service::contacts::sync_contacts(state).await?;
                println!("{} contacts synchronisés", count);
            } else {
                let contacts = meshcore_service::contacts::get_all_contacts(state)?;
                display::contacts(&contacts, json);
            }
        }

        "send" => {
            if tokens.len() < 3 {
                return Err("Usage: send <nom_ou_pubkey> <message>".to_string());
            }
            let pubkey = resolve_contact(state, &tokens[1])?;
            let text = tokens[2..].join(" ");
            let id =
                meshcore_service::messaging::send_direct_message(state, &pubkey, &text).await?;
            println!("Envoyé (id: {})", &id[..8]);
        }

        "channel" => {
            if tokens.len() < 3 {
                return Err("Usage: channel <idx> <message>".to_string());
            }
            let idx: u8 = tokens[1].parse().map_err(|_| "Index invalide")?;
            let text = tokens[2..].join(" ");
            let id = meshcore_service::messaging::send_channel_message(state, idx, &text).await?;
            println!("Canal #{} — envoyé (id: {})", idx, &id[..8]);
        }

        "channels" => {
            let channels = meshcore_service::channels::get_all_channels(state)?;
            display::channels(&channels, json);
        }

        "device" => {
            let info = meshcore_service::device::get_device_info(state).await?;
            display::device_info(&info, json);
        }

        "battery" => {
            let chem = match tokens.get(1).map(|s| s.as_str()) {
                Some("lifepo4") => BatteryChemistry::LiFePO4,
                Some("nimh") => BatteryChemistry::NiMH,
                _ => BatteryChemistry::LiPo,
            };
            let (mv, pct) = meshcore_service::device::get_battery(state, chem).await?;
            display::battery(mv, pct, json);
        }

        "reboot" => {
            meshcore_service::device::reboot(state).await?;
            println!("Dispositif redémarré");
        }

        "repeater" => {
            if tokens.len() < 3 {
                return Err(
                    "Usage: repeater login|logout|status|cli|neighbours <pubkey> [args]"
                        .to_string(),
                );
            }
            match tokens[1].as_str() {
                "login" => {
                    if tokens.len() < 4 {
                        return Err("Usage: repeater login <pubkey> <password>".to_string());
                    }
                    let r =
                        meshcore_service::repeater::login(state, &tokens[2], &tokens[3]).await?;
                    println!("{}", r);
                }
                "logout" => {
                    meshcore_service::repeater::logout(state, &tokens[2]).await?;
                    println!("Déconnecté du repeater");
                }
                "status" => {
                    // Utilise les commandes CLI (plus fiable que le protocole binaire)
                    for cmd in &["ver", "get name", "get radio", "get tx"] {
                        match meshcore_service::repeater::send_cli(state, &tokens[2], cmd).await {
                            Ok(resp) => println!("{}: {}", cmd, resp),
                            Err(e) => println!("{}: erreur - {}", cmd, e),
                        }
                    }
                }
                "cli" => {
                    if tokens.len() < 4 {
                        return Err("Usage: repeater cli <pubkey> <commande>".to_string());
                    }
                    let cmd = tokens[3..].join(" ");
                    let resp =
                        meshcore_service::repeater::send_cli(state, &tokens[2], &cmd).await?;
                    println!("{}", resp);
                }
                "neighbours" => {
                    let n =
                        meshcore_service::repeater::neighbours(state, &tokens[2], 50, 0).await?;
                    display::neighbours(&n, json);
                }
                _ => return Err("Sous-commande repeater inconnue".to_string()),
            }
        }

        "history" => {
            if tokens.len() < 2 {
                return Err("Usage: history <nom_ou_pubkey> [limit]".to_string());
            }
            let pubkey = resolve_contact(state, &tokens[1])?;
            let limit = tokens.get(2).and_then(|l| l.parse().ok()).unwrap_or(20);
            let msgs = meshcore_service::messaging::get_direct_messages(state, &pubkey, limit, 0)?;
            display::messages(&msgs, json);
        }

        _ => {
            return Err(format!(
                "Commande inconnue : {}. Tapez 'help' pour l'aide.",
                tokens[0]
            ));
        }
    }

    Ok(false)
}

/// Résout un nom de contact en clé publique
fn resolve_contact(state: &AppState, name_or_key: &str) -> Result<String, String> {
    // Si ça ressemble à une clé hex (>= 12 chars hex), l'utiliser directement
    if name_or_key.len() >= 12 && name_or_key.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(name_or_key.to_string());
    }
    // Chercher par nom
    let contacts = meshcore_service::contacts::get_all_contacts(state)?;
    let matches: Vec<_> = contacts
        .iter()
        .filter(|c| c.name.to_lowercase().contains(&name_or_key.to_lowercase()))
        .collect();
    match matches.len() {
        0 => Err(format!("Contact '{}' non trouvé", name_or_key)),
        1 => Ok(matches[0].public_key.clone()),
        _ => {
            let names: Vec<_> = matches
                .iter()
                .map(|c| format!("  {} ({})", c.name, &c.public_key[..12]))
                .collect();
            Err(format!(
                "Plusieurs contacts correspondent :\n{}",
                names.join("\n")
            ))
        }
    }
}
