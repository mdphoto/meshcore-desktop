//! Service de connexion : gère le cycle de vie de la connexion au dispositif

use crate::events::AppEvent;
use crate::state::AppState;
use meshcore_rs::events::{EventPayload, EventType};
use meshcore_transport::manager::ConnectionTarget;
use std::collections::HashMap;
use tracing::info;

/// Connecte au dispositif MeshCore, fait le handshake minimal (appstart + set_time)
/// et retourne rapidement. Le chargement des contacts est à faire séparément via sync_contacts.
pub async fn connect(state: &AppState, target: ConnectionTarget) -> Result<(), String> {
    let is_ble = matches!(&target, ConnectionTarget::Ble { .. });
    let default_name = match &target {
        ConnectionTarget::Ble { name_or_addr } => name_or_addr.clone(),
        ConnectionTarget::Serial { port, .. } => port.clone(),
        ConnectionTarget::Tcp { host, port } => format!("{}:{}", host, port),
    };

    let mut conn = state.connection.write().await;
    conn.connect(target).await.map_err(|e| e.to_string())?;

    let mc = conn
        .meshcore()
        .ok_or("Connexion établie mais pas de MeshCore")?;
    let cmds = mc.commands();

    // Stabilisation BLE uniquement (TCP et Serial n'en ont pas besoin)
    if is_ble {
        info!("Attente stabilisation BLE (2s)...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    // Augmenter le timeout par défaut (5s trop court pour BLE multi-hop)
    mc.set_default_timeout(std::time::Duration::from_secs(30))
        .await;

    // APP_START → nom du device
    info!("Handshake: send_appstart...");
    let device_name = match cmds.lock().await.send_appstart().await {
        Ok(si) => {
            info!(
                "Device: {} (pubkey: {}...)",
                si.name,
                meshcore_rs::parsing::hex_encode(&si.public_key[..4])
            );
            if si.name.is_empty() {
                default_name.clone()
            } else {
                si.name.clone()
            }
        }
        Err(e) => {
            info!("send_appstart échoué : {}", e);
            default_name.clone()
        }
    };

    // Sync heure
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32;
    let _ = cmds.lock().await.set_time(now).await;

    // Charger les canaux du device (index 0-7)
    info!("Handshake: get_channels...");
    for idx in 0..8u8 {
        match cmds.lock().await.get_channel(idx).await {
            Ok(ch) if !ch.name.is_empty() => {
                info!("Canal {} : {}", idx, ch.name);
                let stored = meshcore_storage::channels::StoredChannel {
                    idx,
                    name: ch.name,
                    channel_type: "public".to_string(),
                    psk: ch.secret.to_vec(),
                    notifications_enabled: true,
                    unread_count: 0,
                };
                let _ = state
                    .db
                    .with_conn(|c| meshcore_storage::channels::upsert_channel(c, &stored));
            }
            Ok(_) => {}      // Canal vide
            Err(_) => break, // Plus de canaux
        }
    }

    // Auto-fetch messages
    mc.start_auto_message_fetching().await;

    // S'abonner aux événements temps réel
    let state_tx = state.event_tx.clone();

    let tx_dm = state_tx.clone();
    mc.subscribe(EventType::ContactMsgRecv, HashMap::new(), move |event| {
        if let EventPayload::ContactMessage(msg) = &event.payload {
            let _ = tx_dm.send(AppEvent::DirectMessageReceived {
                sender_pubkey: msg.sender_prefix_hex(),
                sender_name: String::new(),
                text: msg.text.clone(),
                snr: msg.snr,
            });
        }
    })
    .await;

    let tx_ch = state_tx.clone();
    mc.subscribe(EventType::ChannelMsgRecv, HashMap::new(), move |event| {
        if let EventPayload::ChannelMessage(msg) = &event.payload {
            let _ = tx_ch.send(AppEvent::ChannelMessageReceived {
                channel_idx: msg.channel_idx,
                sender_name: String::new(),
                text: msg.text.clone(),
            });
        }
    })
    .await;

    let tx_contact = state_tx.clone();
    mc.subscribe(EventType::NewContact, HashMap::new(), move |event| {
        if let EventPayload::Contact(contact) = &event.payload {
            let pubkey = meshcore_rs::parsing::hex_encode(&contact.public_key);
            let _ = tx_contact.send(AppEvent::ContactDiscovered {
                pubkey,
                name: contact.adv_name.clone(),
                node_type: contact.contact_type,
            });
        }
    })
    .await;

    drop(conn);

    // Émettre Connected — le frontend réagit immédiatement
    state.emit(AppEvent::Connected {
        device_name: device_name.clone(),
    });
    info!(
        "Connecté à {} — prêt (contacts à charger via sync)",
        device_name
    );
    Ok(())
}

/// Déconnecte du dispositif
pub async fn disconnect(state: &AppState) -> Result<(), String> {
    let mut conn = state.connection.write().await;
    match tokio::time::timeout(std::time::Duration::from_secs(5), conn.disconnect()).await {
        Ok(Ok(())) => info!("Déconnecté proprement"),
        Ok(Err(e)) => info!("Erreur déconnexion (ignorée) : {}", e),
        Err(_) => info!("Timeout déconnexion — forçage"),
    }
    state.emit(AppEvent::Disconnected);
    Ok(())
}

/// Vérifie l'état de connexion
pub async fn is_connected(state: &AppState) -> bool {
    let conn = state.connection.read().await;
    conn.is_connected().await
}
