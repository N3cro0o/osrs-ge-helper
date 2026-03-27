use iced::{Element, Center, Length};
use iced::widget::{button, column, row, text, center, space, container, combo_box, text_input};
use iced::alignment::Horizontal;

use crate::{Message, MainLayout, CurrentRecipe};
use crate::{APP_PADDING, APP_SPACING, COMBOBOX_MENU_HEIGHT};
use crate::structs;

impl structs::AppPages {
	pub fn calc_sidebar_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		let mut button_vec: Vec<Element<'_, Message>> = vec![];
		for data in state.calc_saved_recipes.iter(){
			let str_offset = data.find(' ').unwrap_or(0);
			let val = data[..str_offset].to_string().parse::<usize>().unwrap_or_default();
			button_vec.push(
				button(text(&data[str_offset + 1..]))
					.on_press_with(move || Message::CalcSelectItem(val))
					.into()
				);
		}
		let button_column = iced::widget::Column::from_vec(button_vec)
			.spacing(APP_SPACING);
		let sidebar = container(
				column![
						text("Saved recipes:").size(22),
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
	
	pub fn calc_body_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		let searchbar: Element<'a, Message>;
		let combo = combo_box(
				&state.combo_data,
				"Select item",
				state.last_item.as_ref(),
				Message::AddItem,
			)
			.menu_height(Length::Fixed(COMBOBOX_MENU_HEIGHT))
			.width(400);
		
		let reset_button = button("Reset")
			.on_press(Message::CalcResetThis);
		
		let save_button = button("Save")
			.on_press_maybe(
				{
					if let CurrentRecipe::Loaded(holder) = &state.calc_curr_recipe {
						let mut answ = None;
						if !holder.is_resources_empty() || !holder.is_products_empty(){
							answ = Some(Message::ShowPopup);
						}
						answ
					}
					else {
						None
					}
				}
			);
		
		if let CurrentRecipe::Loaded(holder) = &state.calc_curr_recipe {
			let resources_panel: Element<'a, Message>;
			let products_panel: Element<'a, Message>;
			
			let cost_text = {
				if !holder.is_resources_empty() && !holder.is_products_empty() {
					let mut prod_cost: i64 = 0;
					let mut resr_cost: i64 = 0;
					for data_tuple in holder.products_iter() {
						let latest_data = match state.latest_ge_data.get_data_by_id(data_tuple.0) {
							Some(data) => data,
							None => continue,
						};
						prod_cost += (latest_data.buy_price().unwrap_or_default() * data_tuple.1) as i64;
					}
					for data_tuple in holder.resources_iter() {
						let latest_data = match state.latest_ge_data.get_data_by_id(data_tuple.0) {
							Some(data) => data,
							None => continue,
						};
						resr_cost += (latest_data.buy_price().unwrap_or_default() * data_tuple.1) as i64;
					}
					Some(text(format!("Profit: {} gp", prod_cost - resr_cost)))
				}
				else {
					None
				}
			};
			
			searchbar = center( row![
						combo,
						save_button,
						space::horizontal(),
						cost_text,
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
			let remove_button_resources = {
				if let Some(item) = &state.last_item {
					Some(
						button("REMOVE")
							.on_press(Message::CalcRemoveResource(item.id))
						)
				}
				else { None }
			};
			let resources_panel_top = row![
					add_button_resources,
					remove_button_resources,
				]
				.padding(APP_PADDING)
				.spacing(APP_SPACING);
			let mut data_vec: Vec<Element<'_, Message>> = vec![];
			for data in holder.resources_iter(){
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
			let remove_button_products = {
				if let Some(item) = &state.last_item {
					Some(
						button("REMOVE")
							.on_press(Message::CalcRemoveProduct(item.id))
						)
				}
				else { None }
			};
			let products_panel_top = row![
					add_button_products,
					remove_button_products,
				]
				.padding(APP_PADDING)
				.spacing(APP_SPACING);
			data_vec = vec![];
			for data in holder.products_iter(){
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
		else {
			searchbar = center( row![
						combo,
						save_button,
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
			
			let main = center(
				column![
						searchbar,
						center(text("Create new or load saved recipe"))
							.style(container::rounded_box)
							.height(Length::FillPortion(10)),
					]
					.align_x(Center)
					.spacing(APP_SPACING)
				);
			main.into()
		}
	}
	
	pub fn calc_overlay_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		use iced::Theme;
		
		let body = column![
				text("Save recipe"),
				text_input("Input recipe name here", &state.extra_string)
					.on_submit(Message::CalcAcceptRecipeName)
					.on_input(Message::ChangeExtraString)
					.padding(APP_PADDING),
				row![
						button("Save")
							.on_press(Message::CalcAcceptRecipeName),
						button("Cancel")
							.on_press(Message::HidePopup),
					]
					.spacing(APP_SPACING)
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
			.height(Length::Fixed(200.0))
			.width(Length::Fixed(400.0))
			.into()
	}
}