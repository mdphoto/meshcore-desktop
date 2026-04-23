use chrono::{DateTime, Local, Utc};

pub fn short_pubkey(pk: &str) -> String {
    let len = pk.len().min(12);
    format!("{}…", &pk[..len])
}

pub fn relative_time(iso: &str) -> String {
    let Ok(parsed) = DateTime::parse_from_rfc3339(iso) else {
        return iso.to_string();
    };
    let parsed: DateTime<Utc> = parsed.with_timezone(&Utc);
    let local: DateTime<Local> = parsed.with_timezone(&Local);
    let now = Local::now();
    if local.date_naive() == now.date_naive() {
        local.format("%H:%M").to_string()
    } else {
        local.format("%d/%m %H:%M").to_string()
    }
}

pub fn node_type_label(node_type: u8) -> &'static str {
    match node_type {
        1 => "client",
        2 => "repeater",
        3 => "room",
        4 => "sensor",
        _ => "?",
    }
}

pub fn node_type_icon(node_type: u8) -> &'static str {
    match node_type {
        1 => "·",
        2 => "R",
        3 => "#",
        4 => "S",
        _ => "?",
    }
}

/// Ordre d'affichage souhaité : repeater → room → client → sensor → inconnu
pub fn node_type_priority(node_type: u8) -> u8 {
    match node_type {
        2 => 0, // repeater
        3 => 1, // room
        1 => 2, // client
        4 => 3, // sensor
        _ => 4, // inconnu
    }
}

/// Frame courante du spinner Braille — calculée à partir de l'horloge pour ne pas stocker d'état
pub fn spinner_frame() -> &'static str {
    const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let idx = (now / 100) as usize % FRAMES.len();
    FRAMES[idx]
}

pub fn node_type_plural(node_type: u8) -> &'static str {
    match node_type {
        1 => "Clients",
        2 => "Repeaters",
        3 => "Rooms",
        4 => "Sensors",
        _ => "Autres",
    }
}
