//! REPL interactif avec rustyline (historique, couleurs, messages entrants)

use crate::commands;
use colored::Colorize;
use meshcore_service::{AppEvent, AppState};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, ExternalPrinter};

/// Lance le REPL interactif
pub async fn run(state: &AppState, json: bool) {
    println!(
        "{}",
        "MeshCore CLI — tapez 'help' pour l'aide, 'quit' pour quitter".dimmed()
    );

    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("Erreur rustyline : {}", e);
            return;
        }
    };

    // Historique
    let history_path = state.data_dir.join("cli_history.txt");
    let _ = rl.load_history(&history_path);

    // Listener de messages entrants (imprime sans casser le prompt)
    let printer = rl.create_external_printer().ok();
    if let Some(mut printer) = printer {
        let mut rx = state.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let line = format_event(&event);
                if !line.is_empty() {
                    let _ = printer.print(line);
                }
            }
        });
    }

    loop {
        let prompt = build_prompt(state).await;
        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(trimmed);

                let tokens: Vec<String> = trimmed.split_whitespace().map(String::from).collect();
                match commands::execute_repl(state, &tokens, json).await {
                    Ok(true) => break,
                    Ok(false) => {}
                    Err(e) => eprintln!("{}", e.red()),
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("Erreur : {}", e);
                break;
            }
        }
    }

    let _ = rl.save_history(&history_path);
    println!("{}", "Au revoir !".dimmed());
}

async fn build_prompt(state: &AppState) -> String {
    let connected = meshcore_service::connection::is_connected(state).await;
    if connected {
        let conn = state.connection.read().await;
        let name = conn
            .target()
            .map(|t| match t {
                meshcore_transport::manager::ConnectionTarget::Serial { port, .. } => port.clone(),
                meshcore_transport::manager::ConnectionTarget::Tcp { host, port } => {
                    format!("{}:{}", host, port)
                }
                meshcore_transport::manager::ConnectionTarget::Ble { name_or_addr } => {
                    name_or_addr.clone()
                }
            })
            .unwrap_or_else(|| "device".to_string());
        format!(
            "{} {} {} ",
            "meshcore".green(),
            format!("[{}]", name).cyan(),
            ">".bold()
        )
    } else {
        format!(
            "{} {} {} ",
            "meshcore".yellow(),
            "[déconnecté]".dimmed(),
            ">".bold()
        )
    }
}

fn format_event(event: &AppEvent) -> String {
    match event {
        AppEvent::DirectMessageReceived {
            sender_pubkey,
            sender_name,
            text,
            ..
        } => {
            let name = if sender_name.is_empty() {
                &sender_pubkey[..12.min(sender_pubkey.len())]
            } else {
                sender_name
            };
            format!("\n{} {}: {}", "[DM]".cyan().bold(), name.bold(), text)
        }
        AppEvent::ChannelMessageReceived {
            channel_idx,
            sender_name,
            text,
        } => {
            format!(
                "\n{} {}: {}",
                format!("[CH#{}]", channel_idx).green().bold(),
                sender_name.bold(),
                text
            )
        }
        AppEvent::Connected { device_name } => {
            format!("\n{}", format!("Connecté à {}", device_name).green())
        }
        AppEvent::Disconnected => format!("\n{}", "Connexion perdue".red()),
        AppEvent::ContactsSynced { count } => {
            format!("\n{}", format!("{} contacts synchronisés", count).dimmed())
        }
        AppEvent::Error { message } => {
            format!("\n{} {}", "[ERREUR]".red().bold(), message)
        }
        _ => String::new(),
    }
}
