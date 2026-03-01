use iced::{Element, Center, Length};
use iced::widget::{button, column, row, text, center, space, container, combo_box, table, scrollable};
use iced::alignment::Horizontal;

use num_format::{Locale, ToFormattedString};

use crate::{Message, MainLayout};
use crate::{APP_PADDING, APP_SPACING, COMBOBOX_MENU_HEIGHT, ALCHEMY_VEC_SIZE};
use crate::osrs::DataHolder;

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
		for data in state.saved_items_item_view.iter(){
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
		//state.best_items_alchemy
		let mut data_vec: Vec<Element<'_, Message>> = vec![];
		let mut text_vec: Vec<Element<'_, Message>> = vec![];
		for data in state.fav_items_alchemy.iter(){
			let diff = match state.best_items_alchemy.iter().find(|item| data.id == item.0) {
				Some(item) => item.1,
				None => 0,
			};
			data_vec.push(text(format!("{}: {diff} gp", data.name())).into());
		}
		let data_column = iced::widget::Column::from_vec(data_vec)
			.spacing(APP_SPACING);
		
		let sidebar = container(
				column![
						text("Favourites:").size(22),
						data_column,
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
				if let Some(_) = state.saved_items_item_view.iter().find(|vec_item| vec_item.id == item.id) {
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
			let new_member_filter = state.combo_current_filter_item_view.clone().unwrap_or_default().flip_member_items();
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
		
		let search_row = center(
				row![
					combo,
					member_button,
					space::horizontal(),
					save_button,
					button("wiki")
						.on_press(Message::OpenWiki),
				]
				.padding([0, 5])
				.spacing(APP_SPACING)
			)
			.align_x(Horizontal::Left)
			.height(Length::FillPortion(1))
			.style(container::rounded_box);
			
		let body = center(
				column![
						text(format!("Value: {}", value.to_formatted_string(&Locale::en))),
						text(format!("Low Alch: {}", loalch.to_formatted_string(&Locale::en))),
						text(format!("High Alch: {}", highalch.to_formatted_string(&Locale::en))),
						space::vertical().height(Length::Fixed(100.0)),
						text(format!("Instant buy: {}", insta_buy.to_formatted_string(&Locale::en))),
						text(format!("Instant sell: {}", insta_sell.to_formatted_string(&Locale::en))),
						text(format!("Daily volume: {}", volume.to_formatted_string(&Locale::en))),
					]
			)
			.height(Length::FillPortion(10))
			.style(container::rounded_box);
		
		let main = center(
			column![
					search_row,
					body,
				]
				.align_x(Center)
				.spacing(APP_SPACING)
			);
		
		// OLD BOX (for future reference)
		// let main = center(
			// column![
					// search_row,
					// space::vertical(),
					// text(format!("Value: {}", value.to_formatted_string(&Locale::en))),
					// text(format!("Low Alch: {}", loalch.to_formatted_string(&Locale::en))),
					// text(format!("High Alch: {}", highalch.to_formatted_string(&Locale::en))),
					// space::vertical(),
					// text(format!("Instant buy: {}", insta_buy.to_formatted_string(&Locale::en))),
					// text(format!("Instant sell: {}", insta_sell.to_formatted_string(&Locale::en))),
					// text(format!("Daily volume: {}", volume.to_formatted_string(&Locale::en))),
					// space::vertical(),
				// ]
				// .align_x(Center)
			// )
			// .padding(APP_PADDING)
			// .style(container::rounded_box);
		main.into()
	}

	fn alch_body_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		let table = {
			let columns = [
				table::column("ID", |data: &(usize, isize)| text(data.0)).width(75),
				table::column("Name", |data: &(usize, isize)| {
					match state.get_item_by_id(data.0) {
						Some(item) => text(item.name()),
						None => text("Cannot get item"),
					}
				}).width(275),
				table::column("Value", |data: &(usize, isize)| {
					match state.get_item_by_id(data.0) {
						Some(item) => text(item.basic_data().0),
						None => text("Cannot get item"),
					}
				}).width(60),
				table::column("Hi-Alch", |data: &(usize, isize)| {
					match state.get_item_by_id(data.0) {
						Some(item) => text(item.basic_data().2),
						None => text("Cannot get item"),
					}
				}).width(60),
				table::column("GE Price", |data: &(usize, isize)| {
					if let Some(ge_data) = state.latest_ge_data.get_data_by_id(data.0) {
						match ge_data.buy_price() {
							Some(v) => text(v),
							None => text("Cannot get price data"),
						}
					}
					else {
						text("Cannot get price data")
					}
					}).width(60),
				table::column("Difference", |data: &(usize, isize)| text(data.1)).width(75),
				table::column("", |data: &(usize, isize)| {
					match state.get_item_by_id(data.0) {
						Some(item) => {
							button("Check")
								.padding([3, 5])
								.on_press(Message::AlchemyCheckItem(item.clone()))
								
						}
						None => {
							button("Check")
								.padding([3, 5])
						}
					}
				}),
				table::column("", |data: &(usize, isize)| {
					match state.get_item_by_id(data.0) {
						Some(item) => {
							let fav_check;
							if let Some(_) = state.fav_items_alchemy.iter().find(|vec_item| vec_item.id == item.id) {
								fav_check = true;
							}
							else {
								fav_check = false;
							}
							button("Fav")
								.padding([3, 5])
								.on_press(Message::AlchemyAddToFav(item.clone()))
								.style(
									if fav_check {
										button::danger
									}
									else {
										button::primary
									})
						}
						None => {
							button("Fav")
								.padding([3, 5])
						}
					}
				})
				];
			if !state.best_items_alchemy.is_empty() {
				let start_offset = (ALCHEMY_VEC_SIZE * state.table_vec_offset) as usize;
				let end_offset = {
					if ALCHEMY_VEC_SIZE + start_offset > state.best_items_alchemy.len() {
						state.best_items_alchemy.len() - ALCHEMY_VEC_SIZE
					}
					else {
						start_offset
					}
				};
				println!("Offsets [{start_offset} - {})", ALCHEMY_VEC_SIZE + end_offset);
				table(columns, &state.best_items_alchemy[0 + start_offset..ALCHEMY_VEC_SIZE + end_offset])
			}
			else {
				println!("ERROR. No alchemy data");
				table(columns, &state.best_items_alchemy)
			}
		};
		let table_buttons = row![
			button("Previous")
				.on_press(Message::AlchemyDecreaseOffset),
			button("Next")
				.on_press(Message::AlchemyIncreaseOffset),
			]
			.padding(APP_PADDING)
			.spacing(200);
		let main = center(
			column![
					scrollable(table),
					table_buttons
				]
				.align_x(Center)
			)
			.padding(APP_PADDING)
			.style(container::rounded_box);
		main.into()
	}

	fn calc_body_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		let searchbar: Element<'a, Message>;
		let resources_panel: Element<'a, Message>;
		let products_panel: Element<'a, Message>;
		
		let combo = combo_box(
				&state.combo_data,
				"Select item",
				state.last_item.as_ref(),
				Message::AddItem,
			)
			.menu_height(Length::Fixed(COMBOBOX_MENU_HEIGHT))
			.width(400);
		let cost_text = {
			if !state.calc_curr_products.is_empty() && !state.calc_curr_resources.is_empty() {
				let mut prod_cost: i64 = 0;
				let mut resr_cost: i64 = 0;
				for data_tuple in state.calc_curr_products.iter() {
					let latest_data = match state.latest_ge_data.get_data_by_id(data_tuple.0) {
						Some(data) => data,
						None => continue,
					};
					prod_cost += latest_data.buy_price().unwrap_or_default() as i64;
				}
				for data_tuple in state.calc_curr_resources.iter() {
					let latest_data = match state.latest_ge_data.get_data_by_id(data_tuple.0) {
						Some(data) => data,
						None => continue,
					};
					resr_cost += latest_data.buy_price().unwrap_or_default() as i64;
				}
				Some(text(format!("Profit: {} gp", prod_cost - resr_cost)))
			}
			else {
				None
			}
		};
		
		let reset_button = button("Reset")
			.on_press(Message::CalcResetThis);
		
		searchbar = center( row![
					combo,
					cost_text,
					space::horizontal(),
					reset_button,
				]
				.spacing(APP_SPACING)
			)
			.height(Length::FillPortion(1))
			.style(container::rounded_box)
			.align_x(Horizontal::Left)
			.padding([0, 5])
			.into();
		// RESOURCES -------------------------
		let add_button_resources = {
			if let Some(item) = &state.last_item {
				Some(
					button("ADD")
						.on_press(Message::CalcAddResource(item.id))
					)
			}
			else { None }
		};
		let resources_panel_top = row![
				add_button_resources,
			]
			.padding(APP_PADDING)
			.spacing(APP_SPACING);
		let mut data_vec: Vec<Element<'_, Message>> = vec![];
		for data in state.calc_curr_resources.iter(){
			let item = match state.get_item_by_id(data.0) {
					Some(item) => item,
					None => continue,
			};
			let latest_data = match state.latest_ge_data.get_data_by_id(data.0) {
				Some(data) => data,
				None => continue,
			};
			data_vec.push(text(format!("{} {}, {} gp", data.1, item.name(), 
					data.1 * latest_data.buy_price().unwrap_or_default())).into());
		}
		let resource_column = iced::widget::Column::from_vec(data_vec)
			.spacing(APP_SPACING);
		resources_panel = center(
				column![
						resources_panel_top,
						center(resource_column),
					]
			)
			.padding(APP_PADDING)
			.style(container::rounded_box)
			.into();
		// PRODUCTS -------------------------
		let add_button_products = {
			if let Some(item) = &state.last_item {
				Some(
					button("ADD")
						.on_press(Message::CalcAddProduct(item.id))
					)
			}
			else { None }
		};
		let products_panel_top = row![
				add_button_products,
			]
			.padding(APP_PADDING)
			.spacing(APP_SPACING);
		data_vec = vec![];
		for data in state.calc_curr_products.iter(){
			let item = match state.get_item_by_id(data.0) {
					Some(item) => item,
					None => continue,
			};
			let latest_data = match state.latest_ge_data.get_data_by_id(data.0) {
				Some(data) => data,
				None => continue,
			};
			data_vec.push(text(format!("{} {}, {} gp", data.1, item.name(), 
					data.1 * latest_data.buy_price().unwrap_or_default())).into());
		}
		let product_column = iced::widget::Column::from_vec(data_vec)
			.spacing(APP_SPACING);
		
		products_panel = center(
				column![
						products_panel_top,
						center(product_column),
					]
			)
			.padding(APP_PADDING)
			.style(container::rounded_box)
			.into();
		
		let main = center(
			column![
					searchbar,
					row![resources_panel, products_panel]
						.spacing(APP_SPACING)
						.height(Length::FillPortion(10)),
				]
				.align_x(Center)
				.spacing(APP_SPACING)
			);
		main.into()
	}
}
