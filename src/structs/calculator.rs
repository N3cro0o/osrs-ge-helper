use iced::{Element, Center, Length};
use iced::widget::{button, column, row, text, center, space, container, combo_box, text_input, text_editor};
use iced::alignment::Horizontal;

use num_format::{Locale, ToFormattedString};

use crate::{Message, MainLayout, CurrentRecipe};
use crate::{APP_PADDING, APP_SPACING, COMBOBOX_MENU_HEIGHT};
use crate::structs;

impl structs::AppPages {
	pub fn calc_sidebar_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		let mut button_vec: Vec<Element<'_, Message>> = vec![];
		for data in state.calc_saved_recipes.iter(){
			let str_offset = data.find(' ').unwrap_or(0);
			let val = data[..str_offset].to_string().parse::<usize>().unwrap_or_default();
			let del_check = state.extra_bool;
			button_vec.push(
				button(text(&data[str_offset + 1..]))
					.on_press_with(move || {
							if del_check {
								Message::CalcDeleteItem(val)
							}
							else {
								Message::CalcSelectItem(val) 
							}
						})
					.into()
				);
		}
		let button_column = iced::widget::Column::from_vec(button_vec)
			.spacing(APP_SPACING);
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
		let delete_button = button("Delete")
			.style(
				if state.extra_bool {
					button::danger
				}
				else {
					button::primary
				}
			)
			.on_press_maybe(
				{
					if state.calc_saved_recipes.is_empty() {
						None
					}
					else {
						let messg;
						if state.extra_bool { messg = Message::CalcDisableDelMode }
						else { messg = Message::CalcEnableDelMode }
						Some(messg)
					}
				}
			);
		let sidebar_top = center( 
			row![
					space::horizontal(),
					save_button,
					delete_button,
					space::horizontal(),
				]
				.spacing(APP_SPACING)
			)
			.height(Length::Fixed(59.0))
			.style(container::rounded_box)
			.align_x(Horizontal::Left)
			.padding([0, 5]);
			
		let sidebar_bottom = container(
				column![
						text("Saved recipes:").size(22),
						button_column,
						space::vertical()
					]
					.width(200)
					.spacing(APP_SPACING)
					.padding(APP_PADDING)
			)
			.style(container::rounded_box)
			.height(Length::Fill);
			
		column![sidebar_top, sidebar_bottom]
			.width(200)
			.max_width(200)
			.spacing(APP_SPACING)
			.into()
	}
	
	pub fn calc_body_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
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
		
		if let CurrentRecipe::Loaded(holder) = &state.calc_curr_recipe {
			let searchbar = center( row![
						combo,
						space::horizontal(),
						reset_button,
					]
					.spacing(APP_SPACING)
				)
				.height(Length::FillPortion(1))
				.style(container::rounded_box)
				.align_x(Horizontal::Left)
				.padding([0, 5]);
			let body = row![
					container(self.calc_body(state, holder))
						.width(Length::FillPortion(2)),
					column![
							self.price_body(state, holder),
							self.desc_body(state, holder),
						]
						.width(Length::Fill)
						.spacing(APP_SPACING),
				]
				.spacing(APP_SPACING)
				.height(Length::FillPortion(10));
				
			let main_body = center(column![searchbar, body].spacing(APP_SPACING));
			main_body.into()
		}
		else {
			let searchbar: Element<'a, Message> = center( row![
						combo,
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
	
	fn calc_body<'a>
		(&'a self, 
		state: &'a MainLayout, 
		holder: &structs::RecipeHolder) -> Element<'a, Message> 
	{
		let resources_panel: Element<'a, Message>;
		let products_panel: Element<'a, Message>;
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
		let resource_info_panel = container(
				row![
						text("TESAT"),
						space::horizontal(),
					]
					.padding(APP_PADDING)
			)
			.height(75)
			.align_y(Center)
			.style(container::rounded_box);
		resources_panel = column![
				center(
					column![
							resources_panel_top,
							center(resource_column),
						]
					)
					.padding(APP_PADDING)
					.style(container::rounded_box),
				resource_info_panel,
			]
			.spacing(APP_SPACING)
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
		let product_info_panel = container(
				row![
						text("TESAT"),
						space::horizontal(),
						button("config")
							.on_press(Message::ChangePage(structs::AppPages::Config)),
					]
			)
			.height(75)
			.align_y(Center)
			.style(container::rounded_box);
		products_panel = column![
			center(
					column![
							products_panel_top,
							center(product_column),
						]
					)
					.padding(APP_PADDING)
					.style(container::rounded_box),
				product_info_panel,
			]
			.spacing(APP_SPACING)
			.into();
		let main = center(
			row![resources_panel, products_panel]
				.spacing(APP_SPACING)
				.height(Length::FillPortion(10))
				.align_y(Center)
			);
		main.into()
	}
	
	fn price_body<'a>
		(&'a self, 
		state: &'a MainLayout, 
		holder: &structs::RecipeHolder) -> Element<'a, Message> 
	{
		let resource_price = {
			if !holder.is_resources_empty() {
				Some(holder.resc_cost)
			}
			else {
				None
			}
		};
		let product_price = {
			if !holder.is_products_empty() {
				Some(holder.prod_cost)
			}
			else {
				None
			}
		};
		let profit_price = {
			if resource_price.is_some() && product_price.is_some(){
				Some(holder.reci_cost)
			}
			else { None }
		};
		let multis = row![
				space::horizontal().width(Length::FillPortion(2)),
				text_input("Input number of repeat operations", &format!("{}", state.calc_price_multi))
					.on_input(Message::CalcChangePriceMultiplier)
					.width(Length::FillPortion(11))
					.align_x(Center),
				space::horizontal().width(Length::FillPortion(2)),
			];
			
		
		center(column![
					text(format!("{} - {} = {} gp",
						product_price.unwrap_or(0).to_formatted_string(&Locale::en), 
						resource_price.unwrap_or(0).to_formatted_string(&Locale::en), 
						profit_price.unwrap_or(0).to_formatted_string(&Locale::en))),
					multis,
				].align_x(Center))
			.style(container::rounded_box).into()
	}
	
	fn desc_body<'a>
		(&'a self, 
		state: &'a MainLayout, 
		_holder: &structs::RecipeHolder) -> Element<'a, Message>
	{
		center(
			text_editor(&state.calc_description)
				.placeholder("Notes...")
				.on_action(Message::CalcChangeItemDesc)
				.padding(APP_PADDING)
				.height(Length::Fill)
				.padding(APP_PADDING))
			.style(container::rounded_box)
			.into()
	}
}