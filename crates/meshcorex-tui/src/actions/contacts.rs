use crate::action::{Action, AsyncResult};
use meshcorex_service::AppState;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub fn spawn_sync(state: Arc<AppState>, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let result = meshcorex_service::contacts::sync_contacts(&state).await;
        let _ = action_tx.send(Action::Async(AsyncResult::ContactsSyncDone(result.clone())));
        if result.is_ok() {
            reload(state, action_tx);
        }
    });
}

pub fn reload(state: Arc<AppState>, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let result = meshcorex_service::contacts::get_all_contacts(&state);
        match result {
            Ok(list) => {
                let _ = action_tx.send(Action::Async(AsyncResult::ContactsReloaded(list)));
            }
            Err(e) => {
                let _ = action_tx.send(Action::Async(AsyncResult::Generic(Err(e))));
            }
        }
    });
}

pub fn sort(list: &mut [meshcorex_storage::models::StoredContact], mode: crate::state::app_state::ContactSortMode) {
    use crate::state::app_state::ContactSortMode;
    use crate::util::format::node_type_priority;
    match mode {
        ContactSortMode::FavoritesAlpha => {
            list.sort_by(|a, b| {
                b.is_favorite
                    .cmp(&a.is_favorite)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });
        }
        ContactSortMode::ByType => {
            list.sort_by(|a, b| {
                node_type_priority(a.node_type)
                    .cmp(&node_type_priority(b.node_type))
                    .then_with(|| b.is_favorite.cmp(&a.is_favorite))
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });
        }
        ContactSortMode::Alpha => {
            list.sort_by_key(|c| c.name.to_lowercase());
        }
    }
}

/// Une ligne affichée dans la tab Contacts — peut être un en-tête de groupe ou un contact réel
#[derive(Debug, Clone)]
pub enum ContactRow {
    /// En-tête de groupe : node_type + nombre total de contacts du groupe
    Header { node_type: u8, count: usize },
    /// Référence vers le contact à l'index donné dans App::contacts
    Contact(usize),
}

pub fn build_rows(
    contacts: &[meshcorex_storage::models::StoredContact],
    mode: crate::state::app_state::ContactSortMode,
    collapsed: &std::collections::HashSet<u8>,
) -> Vec<ContactRow> {
    use crate::state::app_state::ContactSortMode;
    if !matches!(mode, ContactSortMode::ByType) {
        return contacts
            .iter()
            .enumerate()
            .map(|(i, _)| ContactRow::Contact(i))
            .collect();
    }

    let mut rows = Vec::with_capacity(contacts.len() + 5);
    let mut last_type: Option<u8> = None;
    // Compter par type pour afficher le total dans le header
    let mut counts: std::collections::HashMap<u8, usize> =
        std::collections::HashMap::new();
    for c in contacts {
        *counts.entry(c.node_type).or_insert(0) += 1;
    }

    for (i, c) in contacts.iter().enumerate() {
        if last_type != Some(c.node_type) {
            rows.push(ContactRow::Header {
                node_type: c.node_type,
                count: *counts.get(&c.node_type).unwrap_or(&0),
            });
            last_type = Some(c.node_type);
        }
        if !collapsed.contains(&c.node_type) {
            rows.push(ContactRow::Contact(i));
        }
    }
    rows
}

pub fn spawn_toggle_fav(state: Arc<AppState>, pubkey: String, favorite: bool, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        match meshcorex_service::contacts::toggle_favorite(&state, &pubkey, favorite) {
            Ok(()) => reload(state, action_tx),
            Err(e) => {
                let _ = action_tx.send(Action::Async(AsyncResult::Generic(Err(e))));
            }
        }
    });
}

pub fn spawn_delete(state: Arc<AppState>, pubkey: String, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let result = meshcorex_service::contacts::delete_contact(&state, &pubkey).await;
        let msg = match result {
            Ok(()) => Ok(crate::util::i18n::t("toast.contact_deleted")),
            Err(e) => Err(e),
        };
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
        reload(state, action_tx);
    });
}
