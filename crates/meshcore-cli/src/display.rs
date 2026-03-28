//! Affichage formaté et coloré pour la CLI

use colored::Colorize;
use meshcore_service::device::DeviceInfoSummary;
use meshcore_service::repeater::RepeaterNeighbour;
use meshcore_storage::channels::StoredChannel;
use meshcore_storage::models::{StoredContact, StoredMessage};

pub fn contacts(contacts: &[StoredContact], json: bool) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(contacts).unwrap_or_default()
        );
        return;
    }
    if contacts.is_empty() {
        println!("{}", "Aucun contact".dimmed());
        return;
    }
    println!(
        "{:<20} {:<14} {:<10} {:<6} {}",
        "Nom".bold(),
        "Clé".bold(),
        "Type".bold(),
        "Hops".bold(),
        "Vu".bold()
    );
    for c in contacts {
        let type_label = match c.node_type {
            1 => "Client".green(),
            2 => "Repeater".blue(),
            3 => "Room".magenta(),
            4 => "Sensor".yellow(),
            _ => "?".dimmed(),
        };
        let hops = if c.path_len >= 0 {
            c.path_len.to_string()
        } else {
            "—".to_string()
        };
        println!(
            "{:<20} {:<14} {:<10} {:<6} {}",
            c.name,
            &c.public_key[..12.min(c.public_key.len())],
            type_label,
            hops,
            &c.last_seen
        );
    }
    println!("{}", format!("{} contacts", contacts.len()).dimmed());
}

pub fn channels(channels: &[StoredChannel], json: bool) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(channels).unwrap_or_default()
        );
        return;
    }
    if channels.is_empty() {
        println!("{}", "Aucun canal".dimmed());
        return;
    }
    for ch in channels {
        println!(
            "  {} {} {}",
            format!("#{}", ch.idx).cyan(),
            ch.name.bold(),
            if ch.unread_count > 0 {
                format!("({} non lu)", ch.unread_count).red().to_string()
            } else {
                String::new()
            }
        );
    }
}

pub fn device_info(info: &DeviceInfoSummary, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(info).unwrap_or_default());
        return;
    }
    println!("{}", "Dispositif MeshCore".bold().cyan());
    println!("  Nom          : {}", info.name.bold());
    println!("  Clé publique : {}", &info.public_key[..24]);
    println!("  Type         : {}", info.adv_type);
    println!(
        "  TX Power     : {} / {} dBm",
        info.tx_power, info.max_tx_power
    );
    println!(
        "  Radio        : {} MHz, BW={}, SF={}, CR={}",
        info.radio_freq, info.radio_bw, info.sf, info.cr
    );
    if info.lat != 0.0 || info.lon != 0.0 {
        println!("  Position     : {:.5}, {:.5}", info.lat, info.lon);
    }
}

pub fn battery(mv: u16, pct: u8, json: bool) {
    if json {
        println!("{{\"millivolts\":{},\"percent\":{}}}", mv, pct);
        return;
    }
    let bar_len = (pct as usize) / 5;
    let bar = "█".repeat(bar_len);
    let empty = "░".repeat(20 - bar_len);
    let color_bar = if pct > 50 {
        bar.green()
    } else if pct > 20 {
        bar.yellow()
    } else {
        bar.red()
    };
    println!("  Batterie : [{}{}] {}% ({} mV)", color_bar, empty, pct, mv);
}

pub fn neighbours(neighbours: &[RepeaterNeighbour], json: bool) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(neighbours).unwrap_or_default()
        );
        return;
    }
    if neighbours.is_empty() {
        println!("{}", "Aucun voisin".dimmed());
        return;
    }
    for n in neighbours {
        let name = n.name.as_deref().unwrap_or("?");
        let snr_colored = if n.snr > 0.0 {
            format!("{:.1} dB", n.snr).green()
        } else {
            format!("{:.1} dB", n.snr).red()
        };
        let ago = if n.secs_ago < 60 {
            format!("{}s", n.secs_ago)
        } else if n.secs_ago < 3600 {
            format!("{}m", n.secs_ago / 60)
        } else {
            format!("{}h", n.secs_ago / 3600)
        };
        println!(
            "  {} ({}) — SNR: {} — vu il y a {}",
            name.bold(),
            &n.pubkey_hex[..12],
            snr_colored,
            ago
        );
    }
}

pub fn messages(msgs: &[StoredMessage], json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(msgs).unwrap_or_default());
        return;
    }
    if msgs.is_empty() {
        println!("{}", "Aucun message".dimmed());
        return;
    }
    for m in msgs {
        let dir = if m.direction == "outgoing" {
            "→".green()
        } else {
            "←".cyan()
        };
        let sender = if m.direction == "outgoing" {
            "Moi"
        } else {
            &m.sender_name
        };
        let time = &m.timestamp[11..16.min(m.timestamp.len())];
        println!("  {} [{}] {}: {}", dir, time, sender.bold(), m.text);
    }
}

pub fn help() {
    println!("{}", "Commandes MeshCore CLI".bold().cyan());
    println!();
    println!("  {} serial|tcp|ble <args>", "connect".bold());
    println!("  {}", "disconnect".bold());
    println!(
        "  {}                         état de la connexion",
        "status".bold()
    );
    println!();
    println!(
        "  {} [sync]                  lister/synchroniser",
        "contacts".bold()
    );
    println!("  {} <nom|pubkey> <message>  message direct", "send".bold());
    println!(
        "  {} <idx> <message>         message canal",
        "channel".bold()
    );
    println!(
        "  {}                         lister les canaux",
        "channels".bold()
    );
    println!(
        "  {} <nom|pubkey> [limit]    historique messages",
        "history".bold()
    );
    println!();
    println!(
        "  {}                          infos dispositif",
        "device".bold()
    );
    println!("  {} [lipo|lifepo4|nimh]      batterie", "battery".bold());
    println!("  {}                          redémarrer", "reboot".bold());
    println!();
    println!("  {} login <pubkey> <mdp>", "repeater".bold());
    println!("  {} status <pubkey>", "repeater".bold());
    println!("  {} cli <pubkey> <commande>", "repeater".bold());
    println!("  {} neighbours <pubkey>", "repeater".bold());
    println!();
    println!("  {}  {}  {}", "help".bold(), "quit".bold(), "exit".bold());
}
