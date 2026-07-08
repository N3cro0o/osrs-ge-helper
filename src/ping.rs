use crate::log_mess;

pub fn send_notification(item_name: &str, price: usize) {
    log_mess!["The price ({}) of item {} is lower than threshold!", price, item_name];
}
