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

/// Format humain d'un timestamp Unix (en secondes, stocké comme String dans la DB).
/// Rend par exemple « il y a 3 min (2026-04-24 14:29:05) » ou « 2026-04-20 09:14:00 » si ancien.
pub fn format_unix_timestamp(ts_str: &str) -> String {
    let Ok(ts_secs) = ts_str.parse::<i64>() else {
        return if ts_str.is_empty() {
            "(inconnue)".to_string()
        } else {
            format!("({})", ts_str)
        };
    };
    if ts_secs <= 0 {
        return "(inconnue)".to_string();
    }
    let Some(utc) = DateTime::<Utc>::from_timestamp(ts_secs, 0) else {
        return format!("ts={}", ts_secs);
    };
    let local: DateTime<Local> = utc.with_timezone(&Local);
    let now = Local::now();
    let diff = now.signed_duration_since(local);
    let full = local.format("%Y-%m-%d %H:%M:%S").to_string();
    let relative = if diff.num_seconds() < 60 {
        "à l'instant".to_string()
    } else if diff.num_minutes() < 60 {
        format!("il y a {} min", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("il y a {} h", diff.num_hours())
    } else if diff.num_days() < 7 {
        format!("il y a {} j", diff.num_days())
    } else {
        return full;
    };
    format!("{} ({})", relative, full)
}

/// Formate `path_len` (i8) en texte humain :
/// - -1 → « flood (pas de chemin direct connu) »
/// -  0 → « direct »
/// -  N → « N hop(s) »
pub fn format_path_len(path_len: i8) -> String {
    match path_len {
        -1 => "flood (pas de chemin direct connu)".to_string(),
        0 => "direct (0 hop)".to_string(),
        n if n > 0 => format!("{} hop(s)", n),
        other => format!("inconnu ({})", other),
    }
}

/// Décode les bits du flags MeshCore en liste de labels. Bits inconnus affichés en numéro.
/// Référence : les flags MeshCore exposent quelques bits (GPS connu, signé, etc.) mais la
/// liste complète dépend du firmware — on affiche ce qui est raisonnablement documenté.
pub fn format_contact_flags(flags: u8) -> String {
    if flags == 0 {
        return "aucun (0x00)".to_string();
    }
    let mut labels: Vec<String> = Vec::new();
    // Bits usuels observés dans meshcore-rs / MeshCore firmware :
    if flags & 0x01 != 0 {
        labels.push("position connue".to_string());
    }
    if flags & 0x02 != 0 {
        labels.push("signé".to_string());
    }
    if flags & 0x04 != 0 {
        labels.push("bit2".to_string());
    }
    if flags & 0x08 != 0 {
        labels.push("bit3".to_string());
    }
    if flags & 0x10 != 0 {
        labels.push("bit4".to_string());
    }
    if flags & 0x20 != 0 {
        labels.push("bit5".to_string());
    }
    if flags & 0x40 != 0 {
        labels.push("bit6".to_string());
    }
    if flags & 0x80 != 0 {
        labels.push("bit7".to_string());
    }
    format!("0x{:02x} ({}, binaire 0b{:08b})", flags, labels.join(", "), flags)
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

/// Encode des octets en base64 standard (RFC 4648), avec padding.
fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        out.push(ALPHABET[(b0 >> 2) as usize] as char);
        out.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() >= 2 {
            out.push(ALPHABET[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() >= 3 {
            out.push(ALPHABET[(b2 & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Copie une chaîne dans le presse-papier du terminal via OSC 52.
/// Marche avec Terminator, Kitty, Alacritty, iTerm2, Windows Terminal, Wezterm…
/// Avantage : fonctionne aussi en SSH (c'est le terminal local qui copie).
/// Si le terminal ne supporte pas OSC 52 ou bloque la copie, silencieusement ignoré.
pub fn copy_to_clipboard(text: &str) -> std::io::Result<()> {
    use std::io::Write;
    let encoded = base64_encode(text.as_bytes());
    let mut stdout = std::io::stdout().lock();
    write!(stdout, "\x1b]52;c;{}\x07", encoded)?;
    stdout.flush()?;
    Ok(())
}

/// Extrait le nom d'expéditeur depuis le texte d'un message canal.
///
/// Le protocole MeshCore n'inclut pas le nom dans les messages canal binaires
/// (seul le texte est transmis). Par convention, les utilisateurs préfixent
/// leur message avec leur nom : `Alice: bonjour`, `Bob> hello`, `[Charlie] hi`.
///
/// Retourne `Some(nom)` si un pattern reconnu est trouvé en début de message,
/// sinon `None`. Le nom extrait est trimmé et validé (1..=32 chars, charset alnum+_-.).
pub fn extract_sender_name(text: &str) -> Option<String> {
    let text = text.trim_start();

    // Essayons les séparateurs les plus courants
    for sep in [": ", "> ", " : ", " > "] {
        if let Some(pos) = text.find(sep) {
            let candidate = text[..pos].trim();
            if is_valid_sender_name(candidate) {
                return Some(candidate.to_string());
            }
        }
    }

    // Format [Nom]
    if let Some(stripped) = text.strip_prefix('[')
        && let Some(end) = stripped.find("] ")
    {
        let candidate = stripped[..end].trim();
        if is_valid_sender_name(candidate) {
            return Some(candidate.to_string());
        }
    }

    None
}

/// Retire le préfixe « Nom: »/« Nom> »/« [Nom] » du texte quand il correspond au
/// `sender_name` fourni. Évite d'afficher « Alice: Alice: coucou » au rendu quand
/// on a déjà extrait le nom dans la colonne sender_name de la DB.
pub fn strip_sender_prefix<'a>(text: &'a str, sender_name: &str) -> &'a str {
    if sender_name.is_empty() {
        return text;
    }
    let trimmed = text.trim_start();
    for sep in [": ", "> ", " : ", " > "] {
        let prefix = format!("{}{}", sender_name, sep);
        if let Some(rest) = trimmed.strip_prefix(prefix.as_str()) {
            return rest;
        }
    }
    let bracket = format!("[{}] ", sender_name);
    if let Some(rest) = trimmed.strip_prefix(bracket.as_str()) {
        return rest;
    }
    text
}

fn is_valid_sender_name(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    // Limite : 1..=40 graphèmes raisonnables. Accepte emojis, accents, symboles.
    let len = s.chars().count();
    if !(1..=40).contains(&len) {
        return false;
    }
    // Rejette uniquement les séparateurs utilisés pour le parsing + control chars
    !s.chars()
        .any(|c| c.is_control() || matches!(c, ':' | '>' | '\n' | '\r' | '\t'))
}

/// Dérive le PSK 16 octets d'un hashtag room à partir de son nom.
/// Convention MeshCore : `PSK = SHA256("#roomname")[:16]`.
/// Les canaux commençant par `#` sont des hashtag rooms publics : tout le monde
/// qui tape le même nom obtient automatiquement le même PSK.
pub fn derive_hashtag_psk(name: &str) -> [u8; 16] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();
    let mut psk = [0u8; 16];
    psk.copy_from_slice(&hash[..16]);
    psk
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
