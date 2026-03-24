use iced::{Element, Center};
use iced::widget::{button, column, row, text, center, space, container, table, scrollable};

use crate::{Message, MainLayout};
use crate::{APP_PADDING, APP_SPACING, ALCHEMY_VEC_SIZE};
use crate::structs;

impl structs::AppPages {
	pub fn alch_sidebar_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		//state.best_items_alchemy
		let mut data_vec: Vec<Element<'_, Message>> = vec![];
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
}