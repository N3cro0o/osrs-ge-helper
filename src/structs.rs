use iced::{Element, Center, Length};
use iced::widget::{button, column, row, text, center, space, container, combo_box};

use num_format::{Locale, ToFormattedString};

use crate::{Message, MainLayout};
use crate::{APP_PADDING, APP_SPACING, COMBOBOX_MENU_HEIGHT};

#[derive(Debug, Clone, PartialEq)]
pub struct SearchFilter {
	pub only_non_member_items: bool
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum AppPages {
	#[default]
	ItemView,
	Alchemy,
	Calculator,
}

impl SearchFilter {
	pub fn new() -> Self {
		SearchFilter {
			only_non_member_items: false
		}
	}
	
	pub fn flip_member_items(&mut self) -> Self {
		self.only_non_member_items = !self.only_non_member_items;
		self.clone()
	}
}

impl Default for SearchFilter {
	fn default() -> Self {
		SearchFilter::new()
	}
}

impl AppPages {
	pub fn return_current_page_info(&self) -> String {
		match self {
			AppPages::ItemView => format!("Current Page -> Item view"),
			AppPages::Alchemy => format!("Current Page -> Alchemy view"),
			AppPages::Calculator => format!("Current Page -> Recipe calculator"),
		}
	}
	
	pub fn sidebar<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		match self {
			AppPages::ItemView => self.item_sidebar_view(state),
			AppPages::Alchemy => self.alch_sidebar_view(state),
			AppPages::Calculator => self.calc_sidebar_view(state),
		}
	}
	
	pub fn body<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		match self {
			AppPages::ItemView => self.item_body_view(state),
			AppPages::Alchemy => self.alch_body_view(state),
			AppPages::Calculator => self.calc_body_view(state),
		}
	}
	
	fn item_sidebar_view<'a>(&self, state: &'a MainLayout) -> Element<'a, Message> {
		let mut button_vec: Vec<Element<'_, Message>> = vec![];
		for data in state.picked_items.iter(){
			button_vec.push(
				button(text(data.short_description()))
				.on_press_with(|| Message::SelectItem(data.clone()))
				.into()
				);
		}
		let button_column = iced::widget::Column::from_vec(button_vec)
			.spacing(APP_SPACING);
		let sidebar = container(
				column![
						text("Saved items:").size(22),
						button_column,
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

	fn alch_sidebar_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		let sidebar = container(
				column![
						text("Favourites:").size(22),
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

	fn calc_sidebar_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		let sidebar = container(
				column![
						text("Saved recipes:").size(22),
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
	
	fn item_body_view<'a>(&self, state: &'a MainLayout) -> Element<'a, Message> {
		let value;
		let loalch;
		let highalch;
		let insta_sell;
		let insta_buy;
		let volume;
		if let Some(data) = &state.last_item {
			(value, loalch, highalch) = data.basic_data();
			volume = state.item_volume.find(data.id).unwrap_or_default();
		}
		else {
			(value, loalch, highalch, volume) = (0, 0, 0, 0);
		}
		if let Some(data) = &state.last_item_ge {
			(insta_sell, insta_buy) = data.basic_data();
		}
		else {
			(insta_sell, insta_buy) = (0, 0);
		}
		
		let combo = combo_box(
				&state.combo_data,
				"Select item",
				state.last_item.as_ref(),
				Message::AddItem,
			)
			.menu_height(Length::Fixed(COMBOBOX_MENU_HEIGHT))
			.width(400);
		
		let save_button = {
			let label;
			let message;
			let curr_item = state.last_item.clone();
			if let Some(item) = curr_item {
				if let Some(_) = state.picked_items.iter().find(|vec_item| vec_item.id == item.id) {
					label = "forget item";
					message = Message::RemoveItemFromSaved;
				}
				else {
					label = "save item";
				message = Message::AddItemToSaved
				}
			}
			else {
				label = "save item";
				message = Message::AddItemToSaved
			}
			button(label)
				.style(
					if message == Message::RemoveItemFromSaved {
						button::danger
					}
					else {
						button::primary
					})
				.on_press(message)
		};
		let member_button = {
			let label = "non-member items";
			let new_member_filter = state.combo_current_filter.clone().unwrap_or_default().flip_member_items();
			button(label)
				.style(
					if !new_member_filter.only_non_member_items {
						button::danger
					}
					else {
						button::primary
					})
				.on_press(Message::ComboNewFilter(Some(new_member_filter)))
		};
		
		let search_row = row![
				member_button,
				combo,
				save_button,
				button("wiki")
					.on_press(Message::OpenWiki),
			]
			.spacing(APP_SPACING);
		
		let main = center(
			column![
					search_row,
					space::vertical(),
					text(format!("Value: {}", value.to_formatted_string(&Locale::en))),
					text(format!("Low Alch: {}", loalch.to_formatted_string(&Locale::en))),
					text(format!("High Alch: {}", highalch.to_formatted_string(&Locale::en))),
					space::vertical(),
					text(format!("Instant buy: {}", insta_buy.to_formatted_string(&Locale::en))),
					text(format!("Instant sell: {}", insta_sell.to_formatted_string(&Locale::en))),
					text(format!("Daily volume: {}", volume.to_formatted_string(&Locale::en))),
					space::vertical(),
				]
				.align_x(Center)
			)
			.padding(APP_PADDING)
			.style(container::rounded_box);
		main.into()
	}

	fn alch_body_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		let main = center(
			column![
					text("To be implemented")
				]
				.align_x(Center)
			)
			.padding(APP_PADDING)
			.style(container::rounded_box);
		main.into()
	}

	fn calc_body_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		let main = center(
			column![
					text("To be implemented")
				]
				.align_x(Center)
			)
			.padding(APP_PADDING)
			.style(container::rounded_box);
		main.into()
	}
}
