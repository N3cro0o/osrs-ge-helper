use iced::{Element, Length};
use iced::widget::{column, text, center, space, container};

use crate::{Message, MainLayout};
use crate::{APP_PADDING, APP_SPACING};
use crate::structs;

impl structs::AppPages {
	pub fn config_sidebar_view<'a>(&'a self, _state: &'a MainLayout) -> Element<'a, Message> {
		let sidebar = container(
				column![
						text("Options:").size(22),
						space::vertical()
					]
					.spacing(APP_SPACING)
					.padding(APP_PADDING)
			)
			.width(200)
			.max_width(200)
			.style(container::rounded_box);
		sidebar.into()
	}
	
	pub fn config_body_view<'a>(&self, _state: &'a MainLayout) -> Element<'a, Message> {
		let body = center(
				column![
						text("Nothing to see here... for now"),
					]
					.spacing(APP_SPACING)
			)
			.height(Length::FillPortion(10))
			.style(container::rounded_box);
		
		let main = center(body);
		
		main.into()
	}
}