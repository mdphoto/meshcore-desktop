pub mod layout;
pub mod status_bar;
pub mod tab_bar;
pub mod tabs;
pub mod widgets;

use crate::action::Tab;
use crate::app::App;
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App) {
    let areas = layout::split_main(frame.area());

    tab_bar::render(frame, areas.tabs, &app.ui.current_tab);

    match app.ui.current_tab {
        Tab::Connection => tabs::connection::render(frame, areas.body, app),
        Tab::Contacts => tabs::contacts::render(frame, areas.body, app),
        Tab::Chat => tabs::chat::render(frame, areas.body, app),
        Tab::Channels => tabs::channels::render(frame, areas.body, app),
        Tab::Device => tabs::device::render(frame, areas.body, app),
        Tab::Settings => tabs::settings::render(frame, areas.body, app),
    }

    status_bar::render(frame, areas.status, app);

    // Modale (par-dessus tout)
    if let Some(modal) = app.ui.top_modal() {
        // RepeaterAdmin est plein-écran (pas centré)
        if let crate::action::ModalKind::RepeaterAdmin { pubkey, name } = modal {
            widgets::repeater_modal::render(frame, frame.area(), app, pubkey, name);
        } else if let crate::action::ModalKind::ContactInfo { pubkey } = modal {
            if let Some(contact) = app.contacts.iter().find(|c| c.public_key == *pubkey) {
                use crate::util::i18n::t;
                let extras = match contact.node_type {
                    2 => vec![t("contact.info.extra.repeater")],
                    3 => vec![t("contact.info.extra.room")],
                    _ => vec![],
                };
                let extra_hints: Vec<&str> = extras.iter().map(|s| s.as_str()).collect();
                widgets::contact_info::render(
                    frame,
                    frame.area(),
                    contact,
                    &extra_hints,
                );
            }
        } else {
            let channel_new_idx = (0u8..=7)
                .find(|idx| !app.channels.iter().any(|c| c.idx == *idx));
            widgets::modal::render(
                frame,
                frame.area(),
                modal,
                &app.ui.current_tab,
                &app.connection_ui.tcp_input,
                &app.device_ui.name_input,
                app.device_ui.tx_power_draft,
                &app.ui.channel_edit_name,
                app.ui.channel_edit_notifications,
                &app.ui.channel_edit_scope,
                app.ui.channel_edit_field,
                &app.ui.channel_edit_psk_hex,
                &app.ui.channel_new_name,
                &app.ui.channel_new_psk_hex,
                app.ui.channel_new_field,
                channel_new_idx,
                &app.ui.room_login_password,
            );
        }
    }

    // Toasts (au-dessus de tout, y compris les modales, pour garder le feedback visible)
    let live: Vec<&crate::state::Toast> = app.ui.toasts.iter().collect();
    widgets::toast::render(frame, frame.area(), &live);
}
