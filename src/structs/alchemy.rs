use iced::{Element, Center, Length};
use iced::widget::{button, column, row, text, center, space, container, table, scrollable, text_input, checkbox};

use crate::{Message, MainLayout};
use crate::{APP_PADDING, APP_SPACING, ALCHEMY_VEC_SIZE};
use crate::structs;

impl structs::AppPages {
	pub fn alch_sidebar_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		let data_table = {
			let columns = [
					table::column(text("JP2GMD").size(0.001), |fav_data: String| text(fav_data))
				];
			table(columns, state.get_alch_fav_vec())
		};
		
		let sidebar = container(
				column![
						text("Favourites:").size(22),
						data_table,
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
	
	pub fn alch_body_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
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
						state.best_items_alchemy.len()
					}
					else {
						start_offset + ALCHEMY_VEC_SIZE
					}
				};
				table(columns, &state.best_items_alchemy[0 + start_offset.. end_offset])
			}
			else {
				crate::log_err!("No alchemy data");
				table(columns, &state.best_items_alchemy)
			}
		};
		let table_buttons = row![
			button("Previous")
				.on_press(Message::AlchemyDecreaseOffset),
			button("Filters")
				.on_press(Message::ShowPopup),
			button("Next")
				.on_press(Message::AlchemyIncreaseOffset),
			]
			.padding(APP_PADDING)
			.spacing(APP_SPACING);
		let main = center(
			column![
					space::vertical(),
					scrollable(table),
					space::vertical(),
					table_buttons,
					space::vertical().height(Length::Fixed(20.0)),
				]
				.align_x(Center)
			)
			.padding(APP_PADDING)
			.style(container::rounded_box);
		main.into()
	}
	
	pub fn alch_overlay_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		use iced::Theme;

		let body = column![
				space::vertical(),
				text("Alchemy Filters"),
				space::vertical().height(Length::Fixed(20.0)),
				checkbox(state.extra_bool)
					.on_toggle(Message::AlchemyHideLossyItems)
					.label("Hide non-profitable items"),
				checkbox(state.extra_bool_1)
					.on_toggle(Message::AlchemyHideMembersItems)
					.label("Hide non-members items"),
				checkbox(state.extra_bool_2)
					.on_toggle(Message::AlchemyShowFavourites)
					.label("Show only favourites"),
				text("Price range"),
				row![
						text_input("Maximum", &state.extra_string)
								.on_input(Message::AlchemyChangeMaximumPrice)
								.align_x(Center),
						text_input("Minimum", &state.extra_string_1)
								.on_input(Message::AlchemyChangeMinimumPrice)
								.align_x(Center),
					]
					.spacing(APP_SPACING)
					.padding(APP_PADDING),
				text("Volume range"),
				row![
						text_input("Maximum", &state.extra_string_2)
								.on_input(Message::AlchemyChangeMaximumVolume)
								.align_x(Center),
						text_input("Minimum", &state.extra_string_3)
								.on_input(Message::AlchemyChangeMinimumVolume)
								.align_x(Center),
					]
					.spacing(APP_SPACING)
					.padding(APP_PADDING),
				space::vertical(),
			]
			.align_x(Center)
			.padding(APP_PADDING)
			.spacing(APP_SPACING);
		
		center(body)
			.style(|theme: &Theme| {
				let palette = theme.palette();
				let mut style = container::rounded_box(theme);
				style = style.border(iced::border::color(palette.background)
						.rounded(iced::border::Radius::new(5.0))
						.width(5));
				style
			})
			.height(Length::Fixed(350.0))
			.width(Length::Fixed(750.0))
			.into()
	}
}