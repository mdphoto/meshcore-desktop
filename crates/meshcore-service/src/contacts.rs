//! Service de gestion des contacts

use crate::state::AppState;
use crate::events::AppEvent;
use meshcore_storage::contacts as store;
use meshcore_storage::models::StoredContact;
use tracing::info;

/// Synchronise les contacts depuis le dispositif vers la base locale
/// Envoie la commande get_contacts au device (ne lit pas juste le cache)
pub async fn sync_contacts(state: &AppState) -> Result<usize, String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;

    // Envoyer la commande au device (timeout 30s — contacts envoyés un par un en BLE)
    let device_contacts = mc.commands().lock().await
        .get_contacts_with_timeout(0, std::time::Duration::from_secs(30)).await
        .map_err(|e| format!("Erreur get_contacts : {}", e))?;

    let count = device_contacts.len();
    info!("{} contacts reçus du device", count);

    for contact in &device_contacts {
        let pubkey_hex = meshcore_rs::parsing::hex_encode(&contact.public_key);
        // Préserver le statut favori existant
        let existing_fav = state.db.with_conn(|c| store::get_contact(c, &pubkey_hex))
            .ok().flatten().map(|c| c.is_favorite).unwrap_or(false);

        let stored = StoredContact {
            public_key: pubkey_hex,
            name: contact.adv_name.clone(),
            node_type: contact.contact_type,
            flags: contact.flags,
            path: contact.out_path.clone(),
            path_len: contact.path_len,
            lat: meshcore_rs::parsing::from_microdegrees(contact.adv_lat),
            lon: meshcore_rs::parsing::from_microdegrees(contact.adv_lon),
            last_seen: contact.last_advert.to_string(),
            is_favorite: existing_fav,
            group_name: None,
        };
        state.db.with_conn(|c| store::upsert_contact(c, &stored))
            .map_err(|e| e.to_string())?;
    }

    state.emit(AppEvent::ContactsSynced { count });
    info!("{} contacts synchronisés en DB", count);
    Ok(count)
}

pub fn get_all_contacts(state: &AppState) -> Result<Vec<StoredContact>, String> {
    state.db.with_conn(|conn| store::get_all_contacts(conn)).map_err(|e| e.to_string())
}

pub fn get_contact(state: &AppState, public_key: &str) -> Result<Option<StoredContact>, String> {
    state.db.with_conn(|conn| store::get_contact(conn, public_key)).map_err(|e| e.to_string())
}

pub fn toggle_favorite(state: &AppState, public_key: &str, favorite: bool) -> Result<(), String> {
    state.db.with_conn(|conn| store::set_favorite(conn, public_key, favorite)).map_err(|e| e.to_string())
}

pub async fn delete_contact(state: &AppState, public_key: &str) -> Result<(), String> {
    let conn = state.connection.read().await;
    if let Some(mc) = conn.meshcore() {
        let _ = mc.commands().lock().await.remove_contact(public_key).await;
    }
    state.db.with_conn(|conn| store::delete_contact(conn, public_key).map(|_| ()))
        .map_err(|e| e.to_string())
}
