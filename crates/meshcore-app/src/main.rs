//! Point d'entrée de l'application MeshCore Desktop

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use meshcore_service::AppState;
use tauri::{Emitter, Manager};
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,meshcore=debug")),
        )
        .init();

    tracing::info!("Démarrage de MeshCore Desktop");

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Impossible de déterminer le répertoire de données");

            let state = AppState::new(data_dir)
                .expect("Impossible d'initialiser l'état de l'application");

            // Pont événements : broadcast Rust → stockage DB + émission Tauri frontend
            let mut rx = state.subscribe();
            let handle = app.handle().clone();
            let db_path = state.data_dir.join("meshcore.db");
            tauri::async_runtime::spawn(async move {
                // Ouvrir une seconde connexion DB pour les writes async
                let db = meshcore_storage::Database::open(&db_path).ok();
                while let Ok(ref event) = rx.recv().await {
                    // Stocker les messages reçus en DB
                    if let Some(ref db) = db {
                        match event {
                            meshcore_service::AppEvent::DirectMessageReceived { sender_pubkey, sender_name, text, snr } => {
                                let msg = meshcore_storage::models::StoredMessage {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    direction: "incoming".to_string(),
                                    sender_pubkey: Some(sender_pubkey.clone()),
                                    sender_name: sender_name.clone(),
                                    recipient_pubkey: None,
                                    channel_idx: None,
                                    text: text.clone(),
                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                    status: "received".to_string(),
                                    snr: *snr,
                                    rssi: None,
                                    path_len: None,
                                    attempt: 0,
                                    reply_to: None,
                                    reaction: None,
                                };
                                let _ = db.with_conn(|c| meshcore_storage::messages::insert_message(c, &msg));
                            }
                            meshcore_service::AppEvent::ChannelMessageReceived { channel_idx, sender_name, text } => {
                                let msg = meshcore_storage::models::StoredMessage {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    direction: "incoming".to_string(),
                                    sender_pubkey: None,
                                    sender_name: sender_name.clone(),
                                    recipient_pubkey: None,
                                    channel_idx: Some(*channel_idx),
                                    text: text.clone(),
                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                    status: "received".to_string(),
                                    snr: None,
                                    rssi: None,
                                    path_len: None,
                                    attempt: 0,
                                    reply_to: None,
                                    reaction: None,
                                };
                                let _ = db.with_conn(|c| meshcore_storage::messages::insert_message(c, &msg));
                            }
                            _ => {}
                        }
                    }
                    tracing::debug!("Émission événement vers frontend : {:?}", event);
                    if let Err(e) = handle.emit("meshcore-event", event) {
                        tracing::warn!("Erreur émission événement Tauri : {}", e);
                    }
                }
            });

            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Connexion
            commands::connection::connect_serial,
            commands::connection::connect_tcp,
            commands::connection::connect_ble,
            commands::connection::disconnect,
            commands::connection::is_connected,
            commands::connection::scan_ble_devices,
            commands::connection::pair_ble_device,
            commands::connection::is_ble_paired,
            commands::connection::disconnect_by_id,
            commands::connection::set_primary_connection,
            commands::connection::list_connections,
            // Messagerie
            commands::messaging::send_direct_message,
            commands::messaging::send_channel_message,
            commands::messaging::get_direct_messages,
            commands::messaging::get_channel_messages,
            commands::messaging::delete_conversation,
            commands::messaging::search_messages,
            commands::messaging::send_login,
            commands::messaging::get_dm_contact_pubkeys,
            // Contacts
            commands::contacts::sync_contacts,
            commands::contacts::get_all_contacts,
            commands::contacts::toggle_favorite,
            commands::contacts::delete_contact,
            // Canaux
            commands::channels::get_all_channels,
            commands::channels::mark_as_read,
            commands::channels::sync_channel_to_device,
            commands::channels::upsert_channel,
            commands::channels::delete_channel,
            // Appareil
            commands::device::get_device_info,
            commands::device::get_battery,
            commands::device::sync_time,
            commands::device::set_device_name,
            commands::device::set_tx_power,
            commands::device::reboot,
            commands::device::scan_serial_ports,
            // Companions
            commands::companions::get_all_companions,
            commands::companions::save_companion,
            commands::companions::delete_companion,
            // Paramètres
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_all_settings,
            // Analyse
            commands::analysis::analyze_los,
            commands::analysis::get_elevation,
            // Repeater admin
            commands::repeater::repeater_login,
            commands::repeater::repeater_logout,
            commands::repeater::repeater_status,
            commands::repeater::repeater_neighbours,
            commands::repeater::repeater_telemetry,
            commands::repeater::repeater_acl,
            commands::repeater::send_advert,
            commands::repeater::repeater_send_cli,
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors du lancement de l'application");
}
