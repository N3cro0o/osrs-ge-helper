#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{Element, Center, Size, Pixels, Theme, Subscription};
use iced::widget::{button, column, row, text, space, container, combo_box, stack, center};
use iced::widget::text_editor::{self, Content};
use iced::time::{self, Instant, seconds};

use num_format::{Locale, ToFormattedString};

use reqwest::header::USER_AGENT;
use reqwest::blocking::{Client, Response};

mod osrs;
mod structs;
mod files;

use structs::{SearchFilter, AppPages, CurrentRecipe, ItemViewPlot};

pub const BOND_ID: usize = 13190;
pub const USER_AGENT_MESSAGE: &str = "N3cro0oDev (discord: necro0o) - GE Price Calc Prototype";
pub const APP_SPACING: Pixels = Pixels(5.0);
pub const APP_PADDING: Pixels = Pixels(5.0);
pub const COMBOBOX_MENU_HEIGHT: f32 = 300.0;
pub const ALCHEMY_DAILY_VOLUME_LIMIT: usize = 100;
pub const ALCHEMY_VEC_SIZE: usize = 12;

pub struct MainLayout {
	start_time: Instant,
	plotter: ItemViewPlot,
	
    pub _debug_value: bool,
	pub data: Vec<osrs::DataHolder>,
	pub latest_ge_data: osrs::LatestData,
	pub combo_data: combo_box::State<osrs::DataHolder>,
	pub item_volume: osrs::VolumeData,
	pub bond_sell_price: Option<usize>,
	pub last_item: Option<osrs::DataHolder>,
	pub last_item_ge: Option<osrs::GEData>,
	pub title: String,
	pub theme: Option<Theme>,
	pub current_page: AppPages,
	pub popup_ready: bool,
	
	pub saved_items_item_view: Vec<osrs::DataHolder>,
	pub combo_current_filter_item_view: Option<SearchFilter>,
	pub selected_item_timeseries_data: Option<osrs::TimeseriesData>,
	pub selected_timeseries: osrs::Timeseries,
	
	pub fav_items_alchemy: Vec<osrs::DataHolder>,
	pub search_filter_alchemy: Option<SearchFilter>,
	pub best_items_alchemy: Vec<(usize, isize)>,
	pub table_vec_offset: usize,
	
	pub calc_curr_recipe: CurrentRecipe,
	pub calc_saved_recipes: Vec<String>,
	pub calc_description: Content,
	pub calc_price_multi: usize,
	
	pub extra_string: String, // In Calc => recipe label, Alch => max price temp value,
	pub extra_string_1: String, // Alch => min price temp value,
	pub extra_string_2: String, // Alch => max volume temp value,
	pub extra_string_3: String, // Alch => min volume temp value,
	pub extra_bool: bool, // In Calc => delete mode, Alch => hide lossy items
	pub extra_bool_1: bool, // In Alch => hide non-members items
}

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
	Nothing,
	CurrentTesat,
	RefreshData,
	AddItem(osrs::DataHolder),
	SelectItem(osrs::DataHolder),
	ChangePage(AppPages),
	AddItemToSaved,
	RemoveItemFromSaved,
	ComboNewFilter(Option<SearchFilter>),
	AlchNewFilter(Option<SearchFilter>),
	OpenWiki,
	RefreshTick(Instant),
	
	AlchemyIncreaseOffset,
	AlchemyDecreaseOffset,
	AlchemyCheckItem(osrs::DataHolder),
	AlchemyAddToFav(osrs::DataHolder),
	AlchemyChangeMinimumPrice(String),
	AlchemyChangeMaximumPrice(String),
	AlchemyChangeMinimumVolume(String),
	AlchemyChangeMaximumVolume(String),
	AlchemyHideLossyItems(bool),
	AlchemyHideMembersItems(bool),
	
	CalcAddResource(usize),
	CalcRemoveResource(usize),
	CalcAddProduct(usize),
	CalcRemoveProduct(usize),
	CalcResetThis,
	CalcAcceptRecipeName,
	CalcSelectItem(usize),
	CalcDeleteItem(usize),
	CalcEnableDelMode,
	CalcDisableDelMode,
	CalcChangeItemDesc(text_editor::Action),
	CalcChangePriceMultiplier(String),
	
	ChangePlotterTimeseries(osrs::Timeseries),
	ChangeExtraString(String),
	ShowPopup,
	HidePopup,
}

impl MainLayout {
	pub fn new() -> Self {
		let vec = match files::load_recipes_vec() {
			Ok(v) => v,
			Err(err) => {
				log_err!("Cannot get recipe data. {}", err.to_string());
				vec![]
			}
		};
		let theme = Some(Theme::CatppuccinFrappe);
		let mut layout = MainLayout {
			start_time: Instant::now(),
			plotter: ItemViewPlot::default(),
			
			_debug_value: false,
			data: vec![],
			combo_data: combo_box::State::new(vec![]),
			latest_ge_data: osrs::LatestData::default(),
			item_volume: osrs::VolumeData::default(),
			bond_sell_price: None,
			last_item: None,
			last_item_ge: None,
			title: "OSRS GE Calculator".to_string(),
			theme,
			current_page: AppPages::ItemView,
			popup_ready: false,
			
			saved_items_item_view: vec![],
			combo_current_filter_item_view: None,
			selected_item_timeseries_data: None,
			selected_timeseries: osrs::Timeseries::FiveMin,
			
			fav_items_alchemy: vec![],
			search_filter_alchemy: Some(SearchFilter::default()),
			best_items_alchemy: vec![],
			table_vec_offset: 0,
			
			calc_curr_recipe: CurrentRecipe::default(),
			calc_saved_recipes: vec,
			calc_description: Content::new(),
			calc_price_multi: 1,
			
			extra_string: String::new(),
			extra_string_1: String::new(),
			extra_string_2: String::new(),
			extra_string_3: String::new(),
			extra_bool: false,
			extra_bool_1: false,
		};
		layout.update(Message::RefreshData);
		log_mess!("{:#?}", &layout.calc_saved_recipes);
		layout
	}
	
	fn title(&self) -> String {
        format!("N3cro0oDev - {}", self.title)
    }
	
	fn subscription(&self) -> Subscription<Message> {
		time::every(seconds(60)).map(Message::RefreshTick)
	}
	
	pub fn item_view_plot(&self) -> &ItemViewPlot {
		&self.plotter
	}
	
    pub fn view(&self) -> Element<'_, Message> {
		let headline = container(
				row![
						text("Select page:").size(20),
						button(text("Item check"))
							.style(
								if self.current_page == AppPages::ItemView {
									button::danger
								}
								else {
									button::primary
								}
								)
							.on_press_maybe(
								(self.current_page != AppPages::ItemView)
									.then_some(Message::ChangePage(AppPages::ItemView))
								)
							.padding([5,10]),
						button(text("Alchemy"))
							.style(
								if self.current_page == AppPages::Alchemy {
									button::danger
								}
								else {
									button::primary
								}
								)
							.on_press_maybe(
								(self.current_page != AppPages::Alchemy)
									.then_some(Message::ChangePage(AppPages::Alchemy))
								)
							.padding([5,10]),
						button(text("Recipe calculator"))
							.style(
								if self.current_page == AppPages::Calculator {
									button::danger
								}
								else {
									button::primary
								}
								)
							.on_press_maybe(
								(self.current_page != AppPages::Calculator)
									.then_some(Message::ChangePage(AppPages::Calculator))
								)
							.padding([5,10]),
						space::horizontal(),
						text(format!("Bond price: {} gp", self.bond_sell_price.unwrap_or_default().to_formatted_string(&Locale::en))),
						button(text("Refresh data").size(20))
							.padding([5, 10])
							.on_press(Message::RefreshData)
					]
					.padding(APP_PADDING)
					.spacing(APP_SPACING)
					.align_y(Center)
			)
			.style(container::rounded_box);
		
		let side = self.side_body();
		let main = self.main_body();

		let body = container(
				row![
						side,
						main /*.explain(iced::Color::from_rgb(0.0, 1.0, 0.0))*/,
					]
					.spacing(APP_SPACING)
			);
		
		column![headline, body]
			.spacing(APP_SPACING)
			.padding(APP_PADDING)
			.into()
    }

	fn side_body(&self) -> iced::widget::Column<'_, Message> {
		let sidebar = self.current_page.sidebar(self);
		let config_panel = container(
				row![
						text(format!("V. {}", env!("CARGO_PKG_VERSION"))),
						space::horizontal(),
						button("config")
							.on_press(Message::ChangePage(AppPages::Config)),
					]
					.padding(APP_PADDING)
			)
			.width(200)
			.max_width(200)
			.height(75)
			.align_y(Center)
			.style(container::rounded_box);
		let side = column![sidebar, config_panel].spacing(APP_SPACING);
		side
	}

	fn main_body(&self) -> Element<'_, Message> {
		if !self.popup_ready {
			self.current_page.body(self)
		}
		else {
			stack![
				self.current_page.body(self),
				center(self.current_page.overlay(self)),
			]
			.into()
		}
	}

	pub fn update(&mut self, message: Message) {
		match message {
			Message::RefreshData => {
				log_mess!("Get data from OSRS wiki...");
				match self.refresh_data() {
					Ok(size) => log_mess!("Done. Found {} items", size),
					Err(err) => log_err!("{}", err),
				};
				self.bond_sell_price = self.get_price_from_id(BOND_ID).unwrap_or_default().sell_price();
				self.create_combo_box_data();
				self.calculate_best_alchemy();
			}
			
			Message::AddItem(item) => {
				self.select_new_item(&item);
			}
			
			Message::AddItemToSaved => {
				let _ = self.save_current_item();
			}	

			Message::AlchemyAddToFav(item) => {
				let _ = self.alch_save_current_item(item);
			}
			
			Message::OpenWiki => {
				if let Some(item) = self.last_item.clone() {
					if webbrowser::open(&format!("https://oldschool.runescape.wiki/w/{}", item.name)).is_err() {
						log_mess!("Cannot open wiki");
					}
				}
				else {
					log_err!("No item found");
				}
			}
			
			Message::RemoveItemFromSaved => {
				let _ = self.forget_current_item();
			}
			
			Message::SelectItem(item) => {
				self.select_new_item(&item);
				let _ = self.get_timeseries_data(&item);
			}
			
			Message::ChangePage(page) => {
				self.update_page(page);
			}
			
			Message::ComboNewFilter(filter) => {
				self.combo_current_filter_item_view = filter;
				self.create_combo_box_data();
			}
			
			Message::AlchNewFilter(filter) => {
				self.search_filter_alchemy = filter;
				self.calculate_best_alchemy();
			}
			
			Message::RefreshTick(now) => {
				log_mess!("Auto-refresh data from OSRS wiki at {}s ...", now.duration_since(self.start_time).as_secs_f32());
				match self.refresh_data() {
					Ok(size) => log_mess!("Done. Found {} items", size),
					Err(err) => log_err!("{}", err),
				};
				self.bond_sell_price = self.get_price_from_id(BOND_ID).unwrap_or_default().sell_price();
				self.create_combo_box_data();
				self.calculate_best_alchemy();
			}
			
			Message::AlchemyDecreaseOffset => {
				if self.table_vec_offset != 0 {
					self.table_vec_offset -= 1;
				}
				self.update(Message::HidePopup);
			}	
			
			Message::AlchemyIncreaseOffset => {
				if (self.table_vec_offset + 1) * ALCHEMY_VEC_SIZE < self.best_items_alchemy.len() {
					self.table_vec_offset += 1;
				}
				self.update(Message::HidePopup);
			}
			
			Message::AlchemyChangeMinimumPrice(val) => {
				let minimum = match val.clone().parse::<isize>() {
					Ok(d) => d,
					Err(err) => {
						log_err!("{}", err.to_string());
						return;
					}
				};
				if minimum > 0 {
					match &mut self.search_filter_alchemy {
						Some(data) => {
							if minimum as usize <= data.maximum_price {
								let _ = data.change_min_price(minimum as usize);
							}
							self.extra_string_1 = val;
						}
						None => log_mess!("No alchemy filter present. continuing..."),
					}
				}
			}
			
			Message::AlchemyChangeMaximumPrice(val) => {
				let maximum = match val.clone().parse::<isize>() {
					Ok(d) => d,
					Err(err) => {
						log_err!("{}", err.to_string());
						return;
					}
				};
				if maximum > 0 {
					match &mut self.search_filter_alchemy {
						Some(data) => { 
							if maximum as usize >= data.minimum_price {
								let _ = data.change_max_price(maximum as usize); 
							}
							self.extra_string = val;
						}
						None => log_mess!("No alchemy filter present. continuing..."),
					}
				}
			}
			
			Message::AlchemyChangeMinimumVolume(val) => {
				let minimum = match val.clone().parse::<isize>() {
					Ok(d) => d,
					Err(err) => {
						log_err!("{}", err.to_string());
						return;
					}
				};
				if minimum > 0 {
					match &mut self.search_filter_alchemy {
						Some(data) => { 
							if minimum as usize <= data.maximum_volume {
								let _ = data.change_min_volume(minimum as usize); 
							}
							self.extra_string_3 = val;
						}
						
						None => log_mess!("No alchemy filter present. continuing..."),
					}
				}
			}
			
			Message::AlchemyChangeMaximumVolume(val) => {
				let maximum = match val.clone().parse::<isize>() {
					Ok(d) => d,
					Err(err) => {
						log_err!("{}", err.to_string());
						return;
					}
				};
				if maximum > 0 {
					match &mut self.search_filter_alchemy {
						Some(data) => { 
							if data.minimum_volume <= maximum as usize {
								let _ = data.change_max_volume(maximum as usize); 
							}
							self.extra_string_2 = val;
						}
						None => log_mess!("No alchemy filter present. continuing..."),
					}
				}
			}
			
			Message::AlchemyHideLossyItems(b) => {
				self.search_filter_alchemy = match &self.search_filter_alchemy {
					Some(filter) => { 
						self.extra_bool = b;
						Some(filter.clone().change_lossy_items(b))
					}
					None => None,
				};
			}
			
			Message::AlchemyHideMembersItems(b) => {
				self.search_filter_alchemy = match &self.search_filter_alchemy {
					Some(filter) => { 
						self.extra_bool_1 = b;
						Some(filter.clone().change_members_items(b))
					}
					None => None,
				};
			}
			
			Message::AlchemyCheckItem(item) => {
				self.update_page(AppPages::ItemView);
				self.select_new_item(&item);
				let _ = self.get_timeseries_data(&item);
			}
			
			Message::CalcAddResource(item_id) => {
				if let Some(_item) = self.get_item_by_id(item_id) {
					if let CurrentRecipe::Loaded(holder) = &mut self.calc_curr_recipe {
						holder.add_one_to_resources(item_id);
					}
					self.update(Message::HidePopup);
				}
			}
			Message::CalcAddProduct(item_id) => {
				if let Some(_item) = self.get_item_by_id(item_id) {
					if let CurrentRecipe::Loaded(holder) = &mut self.calc_curr_recipe {
						holder.add_one_to_products(item_id);
					}
					self.recalculate_recipe_prices();
					self.update(Message::HidePopup);
				}
			}
						
			Message::CalcRemoveResource(item_id) => {
				if let Some(_item) = self.get_item_by_id(item_id) {
					if let CurrentRecipe::Loaded( holder) = &mut self.calc_curr_recipe {
						if let Some(pos) = holder.resources_iter().position(|data_tuple| item_id == data_tuple.id()) { // check if exists
							holder.remove_one_from_resources(pos);
						}
					}
					self.update(Message::HidePopup);
				}
			}
			
			Message::CalcRemoveProduct(item_id) => {
				if let Some(_item) = self.get_item_by_id(item_id) {
					if let CurrentRecipe::Loaded(holder) = &mut self.calc_curr_recipe {
						if let Some(pos) = holder.products_iter().position(|data_tuple| item_id == data_tuple.id()) { // check if exists
							holder.remove_one_from_products(pos);
						}
					}
					self.update(Message::HidePopup);
				}
			}
			
			Message::ShowPopup => {
				if !self.popup_ready {
					self.popup_ready = true;
				} // Now every button works as a toggle
				else {
					self.popup_ready = false;
					self.extra_stuff_to_do_once_popup_closes();
				}
			}
			
			Message::HidePopup => {
				if self.popup_ready {
					self.popup_ready = false;
					self.extra_stuff_to_do_once_popup_closes();
				}
			}
			
			Message::ChangePlotterTimeseries(timeseries) => {
				self.selected_timeseries = timeseries;
				if let Some(item) = &self.last_item {
					let _ = self.get_timeseries_data(&item.clone());
				}
			}
			
			Message::CalcResetThis => {
				self.calc_curr_recipe = CurrentRecipe::new();
				self.calc_description = Content::new();
				self.update(Message::HidePopup);
			}
			
			Message::CalcChangePriceMultiplier(multi_str) => {
				let multi = match multi_str.parse::<isize>() {
					Ok(d) => d,
					Err(err) => {
						log_err!("{}", err.to_string());
						return;
					}
				};
				if multi > 0 {
					self.calc_price_multi = multi as usize;
					self.recalculate_recipe_prices();
				}
			}
			
			Message::CurrentTesat => {
				self._debug_value = !self._debug_value;
			}
			
			Message::CalcAcceptRecipeName => {
				if let CurrentRecipe::Loaded(holder) = &mut self.calc_curr_recipe {
					holder.set_id(self.calc_saved_recipes.len()).set_label(self.extra_string.clone())
						.set_desc(self.calc_description.text());
					if let Err(err) = files::save_recipe(&holder) {
						log_err!("{}", err.to_string());
						return;
					}
					self.popup_ready = false;
					for data in self.calc_saved_recipes.iter() {
						let str_offset = data.find(' ').unwrap_or(0);
						let val = data[..str_offset].to_string().parse::<usize>().unwrap_or_default();
						if holder.id == val {
							return;
						}
					}
					self.calc_saved_recipes.push(format!("{} {}", holder.id, holder.label.clone()));
				}
			}
			
			Message::CalcSelectItem(id) => {
				let data = match files::load_recipe(id) {
					Ok(d) => d,
					Err(err) => {
						log_err!("{}", err);
						return;
					}
				};
				self.extra_string = data.label.clone();
				self.calc_description = Content::with_text(&data.description);
				self.calc_curr_recipe = CurrentRecipe::from(data);
				self.recalculate_recipe_prices();
			}
			
			Message::CalcDeleteItem(id) => {
				if let Err(err) = files::delete_recipe(id) {
					log_err!("{}", err);
					return;
				}
				self.update(Message::CalcDisableDelMode);
				for i in 0..self.calc_saved_recipes.len() {
					let data = &self.calc_saved_recipes[i];
					let str_offset = data.find(' ').unwrap_or(0);
					let val = data[..str_offset].to_string().parse::<usize>().unwrap_or_default();
					if id == val {
						self.calc_saved_recipes.remove(i);
						break;
					}
				}
				self.calc_description = Content::new();
			}
			
			Message::CalcChangeItemDesc(action) => {
				if let CurrentRecipe::Loaded(_) = self.calc_curr_recipe {
					self.calc_description.perform(action)
				}
			}
			
			Message::CalcEnableDelMode => {
				self.extra_bool = true;
			}
			
			Message::CalcDisableDelMode => {
				self.extra_bool = false;
			}
			
			Message::ChangeExtraString(string) => {
				self.extra_string = string;
			}
			
			_ => {
				log_mess!("Invalid Message detected");
			}
		}
    }
	
	fn update_page(&mut self, page: AppPages) {
		match page {
			AppPages::Alchemy => {
				self.calculate_best_alchemy();
				if let Some(data) = &self.search_filter_alchemy {
					self.extra_string = data.maximum_price.to_string();
					self.extra_string_1 = data.minimum_price.to_string();
					self.extra_string_2 = data.maximum_volume.to_string();
					self.extra_string_3 = data.minimum_volume.to_string();
					self.extra_bool = data.hide_loss_alch;
					self.extra_bool_1 = data.only_non_member_items;
				}
				else {
					self.extra_string.clear();
					self.extra_string_1.clear();
					self.extra_string_2.clear();
					self.extra_string_3.clear();
					self.extra_bool = false;
					self.extra_bool_1 = false;
				}
			}
			AppPages::Calculator => {
				if let CurrentRecipe::Loaded(data) = &self.calc_curr_recipe {
					self.extra_string = data.label.clone();
				}
				else { self.extra_string.clear(); }
				self.extra_string_1.clear();
				self.extra_string_2.clear();
				self.extra_string_3.clear();
				self.extra_bool = false;
				self.extra_bool_1 = false;
			}
			_ => {
				self.last_item = None;
				self.last_item_ge = None;
				self.extra_string.clear();
				self.extra_string_1.clear();
				self.extra_string_2.clear();
				self.extra_string_3.clear();
				self.plotter.reset_data();
				self.extra_bool = false;
				self.extra_bool_1 = false;
			}
		}
		self.current_page = page;
		self.popup_ready = false;
		log_mess!("{}", self.current_page.return_current_page_info());
	}
	
	fn extra_stuff_to_do_once_popup_closes(&mut self) {
		match self.current_page {
			AppPages::Alchemy => {
				self.calculate_best_alchemy();
				self.table_vec_offset = 0;
			}
			
			_ => {
				
			}
		}
	}
	
	fn recalculate_recipe_prices(&mut self) {
		if let CurrentRecipe::Loaded(holder) = &mut self.calc_curr_recipe {
			let mut resr_cost: isize = 0;
			let mut prod_cost: isize = 0;
			for data_tuple in holder.resources_iter() {
				let latest_data = match self.latest_ge_data.get_data_by_id(data_tuple.id()) {
					Some(data) => data,
					None => continue,
				};
				resr_cost += (latest_data.buy_price().unwrap_or_default() * data_tuple.num()) as isize;
			}
			holder.resc_cost = resr_cost * self.calc_price_multi as isize;
			for data_tuple in holder.products_iter() {
				let latest_data = match self.latest_ge_data.get_data_by_id(data_tuple.id()) {
					Some(data) => data,
					None => continue,
				};
				prod_cost += (latest_data.buy_price().unwrap_or_default() * data_tuple.num()) as isize;
			}
			holder.prod_cost = prod_cost * self.calc_price_multi as isize;
			holder.reci_cost = holder.prod_cost - holder.resc_cost;
		}
	}
	
	pub fn get_item_by_id (&self, id: usize) -> Option<&osrs::DataHolder> {
		match self.data.iter().find(|thing| thing.id == id) {
			Some(data) => Some(&data),
			None => None,
		}
	}
	
	fn get_timeseries_data(&mut self, item: &osrs::DataHolder) -> Result<(), String> {
		let url = format!("https://prices.runescape.wiki/api/v1/osrs/timeseries?timestep={}&id={}", self.selected_timeseries, item.id);
		let response = match self.fetch_get_data(&url) {
			Ok(resp) => resp,
			Err(err) => {
				return Err(err.to_string());
			}
		};
		if !response.status().is_success(){
			return Err(format!("Response failed. {}", response.status()));
		}
		let body = response.text().unwrap();
		let data = match serde_json::from_str::<osrs::TimeseriesData>(&body){
			Ok(data) => data,
			Err(err) => return Err(format!("{}\n{}", err.to_string(), body)),
		};
		self.plotter.change_label(item.name());
		self.plotter.update_data(data);
		// self.selected_item_timeseries_data = Some(data);
		Ok(())
	}
	
	fn calculate_best_alchemy(&mut self) {
		let options = self.create_filtered_vec(&self.search_filter_alchemy);
		let mut output: Vec<(usize, isize)> = vec![];
		for item in options {
			// calc alchemy cost
			let data = item.basic_data().2;
			let value = match self.latest_ge_data.get_data_by_id(item.id) {
				Some(data) => {
					match data.buy_price() {
						Some(val) => val,
						None => continue,
					}
				}
				None => continue,
			};
			let diff: isize = data as isize - value as isize;
			output.push((item.id, diff));
		}
		if output.is_empty() {
			log_err!("ERROR. No alchemy data");
			return;
		}
		output.sort_by(|a, b| b.1.cmp(&a.1));
		self.best_items_alchemy = output;
	}
	
	pub fn create_filtered_vec(&self, filter: &Option<SearchFilter>) -> Vec<osrs::DataHolder> {
		let mut new_vec = vec![];
		for item in self.data.iter() {
			let value = match self.latest_ge_data.get_data_by_id(item.id) {
				Some(data) => {
					match data.buy_price() {
						Some(val) => val,
						None => continue,
					}
				}
				None => continue,
			};
			let volume = match self.item_volume.find(item.id) {
				Some(data) => data,
				None => continue,
			};
			if item.check_filter(filter, value, volume) {
				new_vec.push(item.clone());
			}
		}
		log_mess!("Size of new vector: {}", new_vec.len());
		new_vec
	}
	
	fn create_combo_box_data(&mut self) {
		let new_vec = self.create_filtered_vec(&self.combo_current_filter_item_view);
		self.combo_data = combo_box::State::new(new_vec);
	}
	
	fn save_current_item(&mut self) -> Result<(), (u8, String)> {
		if let None = self.last_item {
			return Err((1, String::from("No selected item")));
		}
		self.saved_items_item_view.push(self.last_item.clone().unwrap());
		Ok(())
	}	
	
	fn alch_save_current_item(&mut self, item: osrs::DataHolder) -> Result<bool, String> {
		// Check for Item
			// Add Item -> true
			// Forget Item -> false
		if let Some(pos) = self.fav_items_alchemy.iter().position(|fav_item| item == *fav_item) {
			let _ = self.fav_items_alchemy.remove(pos);
			Ok(false)
		}
		else {
			self.fav_items_alchemy.push(item);
			Ok(true)
		}
	}
	
	fn forget_current_item(&mut self) -> Result<(), (u8, String)> {
		if let None = self.last_item {
			return Err((1, String::from("No selected item")));
		}
		let last_item = self.last_item.clone().unwrap(); 
		if let Some(pos) = self.saved_items_item_view.iter().position(|vec_item| vec_item.id == last_item.id) {
			let _ = self.saved_items_item_view.remove(pos);
		}
		Ok(())
	}
	
	fn select_new_item(&mut self, item: &osrs::DataHolder){
		log_mess!("Selected new item: {}", item.id);
		match self.get_price_from_id(item.id) {
			Ok(data) => {
				self.last_item_ge = Some(data);
				self.last_item = Some(item.clone());
			}
			Err(err) => {
				log_err!("{}", err);
			}
		}
	}
	
	fn get_price_from_id(&self, id: usize) -> Result<osrs::GEData, String> {
		// let response = match self.fetch_get_data(&format!("https://prices.runescape.wiki/api/v1/osrs/latest?id={}", id)) {
			// Ok(data) => data,
			// Err(err) => {
				// return Err(err.to_string());
			// }
		// };
		// let body = response.text().unwrap();
		// let index = body.find(&id.to_string()).unwrap();
		// let body = &body[index + &id.to_string().len() + 2 .. body.len() - 2];
		// match serde_json::from_str::<osrs::GEData>(&body){
			// Ok(data) => Ok(data),
			// Err(err) => Err(err.to_string()),
		// }
		self.latest_ge_data.get_data_by_id(id).ok_or(format!("Cannot find desired item {id}"))
	}
	
	fn refresh_data(&mut self) -> Result<usize, String> {
		let result = self.refresh_item_data();
		if let Ok(_) = result {
			self.refresh_volume_data()?;
			self.refresh_latest_data()?;
			self.refresh_plotter_data()?;
		}
		result
	}
	
	fn refresh_plotter_data(&mut self) -> Result<(), String> {
		if let Some(item) = self.last_item.clone() {
			self.get_timeseries_data(&item)
		}
		else {
			Ok(())
		}
	}
	
	fn refresh_item_data(&mut self) -> Result<usize, String> {
		let response = match self.fetch_get_data("https://prices.runescape.wiki/api/v1/osrs/mapping") {
			Ok(resp) => resp,
			Err(err) => {
				return Err(err.to_string());
			}
		};
		if !response.status().is_success(){
			return Err(format!("Response failed. {}", response.status()));
		}
		let mut data = match response.json::<Vec<osrs::DataHolder>>() {
			Ok(vec) => vec,
			Err(err) => {
				return Err(err.to_string());
			}
		};
		let len = data.len();
		data.sort_by(|a, b| a.id.cmp(&b.id));
		self.data = data;
		Ok(len)
	}
	
	fn refresh_volume_data(&mut self) -> Result<(), String> {
		let response = match self.fetch_get_data("https://prices.runescape.wiki/api/v1/osrs/volumes") {
			Ok(resp) => resp,
			Err(err) => {
				return Err(err.to_string());
			}
		};
		if !response.status().is_success(){
			return Err(format!("Response failed. {}", response.status()));
		}
		let body = response.text().unwrap();
		let data = match serde_json::from_str::<osrs::VolumeData>(&body) {
			Ok(vec) => vec,
			Err(err) => {
				return Err(err.to_string());
			}
		};
		self.item_volume = data;
		Ok(())
	}	
	
	fn refresh_latest_data(&mut self) -> Result<(), String> {
		let response = match self.fetch_get_data("https://prices.runescape.wiki/api/v1/osrs/latest") {
			Ok(resp) => resp,
			Err(err) => {
				return Err(err.to_string());
			}
		};
		if !response.status().is_success(){
			return Err(format!("Response failed. {}", response.status()));
		}
		let body = response.text().unwrap();
		let data = match serde_json::from_str::<osrs::LatestData>(&body) {
			Ok(vec) => vec,
			Err(err) => {
				return Err(err.to_string());
			}
		};
		self.latest_ge_data = data;
		Ok(())
	}
	
	fn fetch_get_data(&self, url: &str) -> reqwest::Result<Response> {
		let client = Client::new();
		let response = client.get(url)
			.header(USER_AGENT, "N3cro0oDev (necro0o) - GE Price Calc Prototype")
			.send();
		response
	}
	
	fn theme(&self) -> Option<Theme> {
		self.theme.clone()
	}
}

impl Default for MainLayout {
	fn default() -> Self {
		MainLayout::new()
	}
}

fn main() -> iced::Result<> {
	if let Err(err) = files::setup_logger() { return Err(iced::Error::ExecutorCreationFailed(err)) }; // Good enough for now, I believe more Errors should be added to iced::Error 
	log_mess!["INIT APP"];
	
	let mut window_settings = iced::window::Settings::default();
	window_settings.min_size = Some(Size::new(1280.0,720.0));
	window_settings.size = Size::new(1280.0,720.0);
	window_settings.resizable = true;
	
	let app = iced::application(MainLayout::default, MainLayout::update, MainLayout::view)
		.window(window_settings)
		.theme(MainLayout::theme)
		.centered()
		.subscription(MainLayout::subscription)
		.title(MainLayout::title);
	let r = app.run();
	r
}