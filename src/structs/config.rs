use iced::{Element, Length};
use iced::widget::{column, text, center, space, container, button};

use crate::{Message, MainLayout};
use crate::{APP_PADDING, APP_SPACING};
use crate::structs::{self, ConfigPages};

impl structs::AppPages {
	pub fn config_sidebar_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		let sidebar = container(
				column![
						text("Options:").size(22),
						button("General")
							.on_press_maybe(
								(state.config_curr_page != ConfigPages::AppSettings)
									.then_some(Message::ChangeConfigPage(ConfigPages::AppSettings))
								)
							.style(
								if state.config_curr_page == ConfigPages::AppSettings {
									button::danger
								}
								else {
									button::primary
								}
								),
						button("Window")
							.on_press_maybe(
								(state.config_curr_page != ConfigPages::WindowSettings)
									.then_some(Message::ChangeConfigPage(ConfigPages::WindowSettings))
								)
							.style(
								if state.config_curr_page == ConfigPages::WindowSettings {
									button::danger
								}
								else {
									button::primary
								}
								),
						button("Customisation")
							.on_press_maybe(
								(state.config_curr_page != ConfigPages::Customization)
									.then_some(Message::ChangeConfigPage(ConfigPages::Customization))
								)
							.style(
								if state.config_curr_page == ConfigPages::Customization {
									button::danger
								}
								else {
									button::primary
								}
								),
						button("Notifcations")
							.on_press_maybe(
								(state.config_curr_page != ConfigPages::PingSettings)
									.then_some(Message::ChangeConfigPage(ConfigPages::PingSettings))
								)
							.style(
								if state.config_curr_page == ConfigPages::PingSettings {
									button::danger
								}
								else {
									button::primary
								}
								),
						space::vertical(),
						button("Credits")
							.on_press_maybe(
								(state.config_curr_page != ConfigPages::Credits)
									.then_some(Message::ChangeConfigPage(ConfigPages::Credits))
								)
							.style(
								if state.config_curr_page == ConfigPages::Credits {
									button::danger
								}
								else {
									button::primary
								}
								),
					]
					.spacing(APP_SPACING)
					.padding(APP_PADDING)
			)
			.width(200)
			.max_width(200)
			.style(container::rounded_box);
		sidebar.into()
	}
	
	pub fn config_body_view<'a>(&self, state: &'a MainLayout) -> Element<'a, Message> {
		let body = match state.config_curr_page {
			ConfigPages::AppSettings => self.app_settings_body(state),
			ConfigPages::WindowSettings => self.window_settings_body(state),
			ConfigPages::Customization => self.customization_settings_body(state),
			ConfigPages::PingSettings => self.ping_settings_body(state),
			ConfigPages::Credits => self.credits_body(state),
		};
		
		let main = center(body)
			.style(container::rounded_box);
		main.into()
	}
	
	fn app_settings_body<'a>(&self, _state: &'a MainLayout) -> Element<'a, Message>  {
		column![
				text("Nothing"),
			]
			.spacing(APP_SPACING)
			.into()
	}
	
	fn window_settings_body<'a>(&self, _state: &'a MainLayout) -> Element<'a, Message> {
		column![
				text("to see"),
			]
			.spacing(APP_SPACING)
			.into()
	}
	
	fn ping_settings_body<'a>(&self, _state: &'a MainLayout) -> Element<'a, Message> {
		column![
				text("for now"),
			]
			.spacing(APP_SPACING)
			.into()
	}
	
	fn customization_settings_body<'a>(&self, _state: &'a MainLayout) -> Element<'a, Message> {
		column![
				text("here..."),
			]
			.spacing(APP_SPACING)
			.into()
	}
	
	fn credits_body<'a>(&self, _state: &'a MainLayout) -> Element<'a, Message> {
		column![
				text("MONEY"),
			]
			.spacing(APP_SPACING)
			.into()
	}
}