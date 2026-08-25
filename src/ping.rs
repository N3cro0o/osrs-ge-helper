use iced::window::{UserAttention, request_user_attention, latest};
use iced::Task;

use crate::{log_mess, log_err};
use crate::message::Message;
use crate::structs::ConfigSettingsNotifications as Settings;

/// Main function used to handle sending notifications. Before preparing notifications tasks and
/// function calls, checks if notification functionality is enabled.
pub fn send_notification(item_name: &str, price: usize, notif_settings: &Settings) -> Task<Message> {
    log_mess!["The price ({}) of item {} is lower than threshold!", price, item_name];
    if notif_settings.enable {
        if notif_settings.sound_enable {
            if let Err(err) = crate::audio::play_audio("ping", notif_settings.sound_volume) {
                log_err!["Cannot play audio file: {}", err];
            }
        }
        latest().and_then(move |id| request_user_attention::<Message>(id, Some(UserAttention::Informational)))
    }
    else {
        Task::none()
    }
}


