use iced::window::{UserAttention, request_user_attention, latest};
use iced::Task;

use crate::log_mess;
use crate::message::Message;

/// Main function used to handle sending notifications. For now only UserAttention type of
/// notifications are implemented.
pub fn send_notification(item_name: &str, price: usize) -> Task<Message> {
    log_mess!["The price ({}) of item {} is lower than threshold!", price, item_name];
    latest().and_then(move |id| request_user_attention::<Message>(id, Some(UserAttention::Informational)))
}
