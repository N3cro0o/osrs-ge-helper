use iced::{Element, Center, Length};
use iced::widget::{button, column, row, text, center, space, container, combo_box, text_input};
use iced::alignment::Horizontal;

use num_format::{Locale, ToFormattedString};

use crate::{Message, MainLayout};
use crate::{APP_PADDING, APP_SPACING, COMBOBOX_MENU_HEIGHT, UPPER_BAR_HEIGHT};
use crate::osrs;
use crate::structs;

impl structs::AppPages {
	pub fn item_sidebar_view<'a>(&self, state: &'a MainLayout) -> Element<'a, Message> {
		let mut button_vec: Vec<Element<'_, Message>> = vec![];
		for data in state.saved_items_item_view.iter(){
			button_vec.push(
				button(text(data.short_description()))
				.on_press_with(|| Message::SelectItem(data.clone()))
				.width(Length::Fixed(500.0))
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
	
	pub fn item_body_view<'a>(&self, state: &'a MainLayout) -> Element<'a, Message> {
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
				Message::SelectItem,
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
    let fav_price_input = text_input("Input price", &state.extra_string_3)
        .on_submit(Message::UpdatePriceThershold)
        .on_input(Message::ChangeExtraString3);
		
		let search_row = center(
				row![
					combo,
					member_button,
					space::horizontal(),
          if state.selected_item_favourite { Some(fav_price_input) } else { None },
					save_button,
					button("wiki")
						.on_press(Message::OpenWiki),
				]
				.padding([0, 5])
				.spacing(APP_SPACING)
			)
			.align_x(Horizontal::Left)
			.height(Length::Fixed(UPPER_BAR_HEIGHT))
			.style(container::rounded_box);
		
		let body_data_row = row![
				column![
						text(format!("Value: {}", value.to_formatted_string(&Locale::en))),
						text(format!("Low Alch: {}", loalch.to_formatted_string(&Locale::en))),
						text(format!("High Alch: {}", highalch.to_formatted_string(&Locale::en))),
					],
					space::horizontal().width(Length::Fixed(200.0)),
				column![
						text(format!("Instant buy: {}", insta_buy.to_formatted_string(&Locale::en))),
						text(format!("Instant sell: {}", insta_sell.to_formatted_string(&Locale::en))),
						text(format!("Daily volume: {}", volume.to_formatted_string(&Locale::en))),
					],
			]
			.spacing(APP_SPACING);
		
		let bttn_label_vec = vec!["1 day", "7 days", "30 days", "1 year"];
		let mut bttn_vec: Vec<Element<'_, Message>> = vec![];
		{
			let mut index = 0;
			for series in osrs::Timeseries::ALL() {
				let mut bttn = button(bttn_label_vec[index]);
				if series == state.selected_timeseries {
					bttn = bttn.style(button::danger).on_press_maybe(None);
				}
				else {
					bttn = bttn.on_press(Message::ChangePlotterTimeseries(series));
				}
				bttn_vec.push(bttn.into());
				index += 1;
			}
		}
		let plotter:Option<Element<'_, Message>> = if state.last_item_ge.is_some() { Some(state.item_view_plot().view().into()) }
				else { Some(space::vertical().into()) };
			
		let body_center = column![
				row![
						space::horizontal(),
						iced::widget::Row::from_vec(bttn_vec).spacing(APP_SPACING),
						space::horizontal(),
					]
					.spacing(APP_SPACING),
				plotter,
			];
		
		let body = center(
				column![
						center(body_data_row)
							.height(Length::FillPortion(1)),
						center(body_center)
							.height(Length::FillPortion(5)),
					]
					.spacing(APP_SPACING)
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
	
}
