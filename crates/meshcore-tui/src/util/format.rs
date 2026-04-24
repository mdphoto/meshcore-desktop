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
