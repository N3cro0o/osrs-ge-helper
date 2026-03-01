use iced::{Element, Center, Size, Pixels, Theme, Subscription};
use iced::widget::{button, column, row, text, space, container, combo_box};
use iced::time::{self, Instant, seconds};

use num_format::{Locale, ToFormattedString};

use reqwest::header::USER_AGENT;
use reqwest::blocking::{Client, Response};

mod osrs;
mod structs;

use structs::{SearchFilter, AppPages};

pub const APP_VERSION: &str = "0.2.";
pub const BOND_ID: usize = 13190;
pub const USER_AGENT_MESSAGE: &str = "N3cro0oDev (discord: necro0o) - GE Price Calc Prototype";
pub const APP_SPACING: Pixels = Pixels(5.0);
pub const APP_PADDING: Pixels = Pixels(5.0);
pub const COMBOBOX_MENU_HEIGHT: f32 = 300.0;
pub const ALCHEMY_DAILY_VOLUME_LIMIT: usize = 100;
pub const ALCHEMY_VEC_SIZE: usize = 12;

pub struct MainLayout {
	start_time: Instant,
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
	
	pub saved_items_item_view: Vec<osrs::DataHolder>,
	pub combo_current_filter_item_view: Option<SearchFilter>,
	
	pub fav_items_alchemy: Vec<osrs::DataHolder>,
	pub search_filter_alchemy: Option<SearchFilter>,
	pub best_items_alchemy: Vec<(usize, isize)>,
	pub table_vec_offset: usize,
	
	pub calc_curr_resources: Vec<(usize, usize)>, // 0 - ID, 1 - how many
	pub calc_curr_products: Vec<(usize, usize)>, // 0 - ID, 1 - how many
}

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
	Nothing,
    Increment,
    Decrement,
	RefreshData,
	AddItem(osrs::DataHolder),
	SelectItem(osrs::DataHolder),
	ChangePage(AppPages),
	AddItemToSaved,
	RemoveItemFromSaved,
	ComboNewFilter(Option<SearchFilter>),
	OpenWiki,
	RefreshTick(Instant),
	AlchemyIncreaseOffset,
	AlchemyDecreaseOffset,
	AlchemyCheckItem(osrs::DataHolder),
	AlchemyAddToFav(osrs::DataHolder),
	CalcAddResource(usize),
	CalcAddProduct(usize),
	CalcResetThis,
}

impl MainLayout {
	pub fn new() -> Self {
		let vec: Vec<osrs::DataHolder> = vec![];
		let theme = Some(Theme::CatppuccinFrappe);
		let mut layout = MainLayout {
			start_time: Instant::now(),
			_debug_value: false,
			data: vec![],
			combo_data: combo_box::State::new(vec),
			latest_ge_data: osrs::LatestData::default(),
			item_volume: osrs::VolumeData::default(),
			bond_sell_price: None,
			last_item: None,
			last_item_ge: None,
			title: "OSRS GE Calculator".to_string(),
			theme,
			current_page: AppPages::ItemView,
			
			saved_items_item_view: vec![],
			combo_current_filter_item_view: None,
			
			fav_items_alchemy: vec![],
			search_filter_alchemy: None,
			best_items_alchemy: vec![],
			table_vec_offset: 0,
			
			calc_curr_resources: vec![],
			calc_curr_products: vec![],
		};
		layout.update(Message::RefreshData);
		layout
	}
	
	fn title(&self) -> String {
        format!("N3cro0oDev - {}", self.title)
    }
	
	fn subscription(&self) -> Subscription<Message> {
		time::every(seconds(60)).map(Message::RefreshTick)
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
						main,
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
						text(format!("v {APP_VERSION}")),
						space::horizontal(),
						button("config"),
					]
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
		self.current_page.body(self)
	}

	pub fn update(&mut self, message: Message) {
		match message {
			Message::RefreshData => {
				println!("Get data from OSRS wiki...");
				match self.refresh_data() {
					Ok(size) => println!("Done. Found {size} items"),
					Err(err) => println!("{err}"),
				};
				self.bond_sell_price = self.get_price_from_id(BOND_ID).unwrap_or_default().sell_price();
				self._debug_value = !self._debug_value;
				self.create_combo_box_data();
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
						println!("Cannot open wiki");
					}
				}
				else {
					println!("No item found");
				}
			}
			
			Message::RemoveItemFromSaved => {
				let _ = self.forget_current_item();
			}
			
			Message::SelectItem(item) => {
				self.select_new_item(&item);
			}
			
			Message::ChangePage(page) => {
				self.update_page(page);
			}
			
			Message::ComboNewFilter(filter) => {
				self.combo_current_filter_item_view = filter;
				self.create_combo_box_data();
			}
			
			Message::RefreshTick(now) => {
				println!("Auto-refresh data from OSRS wiki at {}s ...", now.duration_since(self.start_time).as_secs_f32());
				match self.refresh_data() {
					Ok(size) => println!("Done. Found {size} items"),
					Err(err) => println!("{err}"),
				};
				self.bond_sell_price = self.get_price_from_id(BOND_ID).unwrap_or_default().sell_price();
				self._debug_value = !self._debug_value;
				self.create_combo_box_data();
			}
			
			Message::AlchemyDecreaseOffset => {
				if self.table_vec_offset != 0 {
					self.table_vec_offset -= 1;
				}
				println!("Offset {}, size {}", self.table_vec_offset * ALCHEMY_VEC_SIZE, self.best_items_alchemy.len());
			}	
			
			Message::AlchemyIncreaseOffset => {
				if (self.table_vec_offset + 1) * ALCHEMY_VEC_SIZE < self.best_items_alchemy.len() {
					self.table_vec_offset += 1;
				}
				println!("Offset {}, size {}", self.table_vec_offset * ALCHEMY_VEC_SIZE, self.best_items_alchemy.len());
			}
			
			Message::AlchemyCheckItem(item) => {
				self.update_page(AppPages::ItemView);
				self.select_new_item(&item);
			}
			
			Message::CalcAddResource(item_id) => {
				if let Some(item) = self.get_item_by_id(item_id) {
					if let Some(pos) = self.calc_curr_resources.iter().position(|data_tuple| item_id == data_tuple.0) {
						self.calc_curr_resources[pos].1 += 1;
					}
					else {
						self.calc_curr_resources.push((item_id, 1));
					}
				}
			}
			Message::CalcAddProduct(item_id) => {
				if let Some(item) = self.get_item_by_id(item_id) {
					if let Some(pos) = self.calc_curr_products.iter().position(|data_tuple| item_id == data_tuple.0) {
						self.calc_curr_products[pos].1 += 1;
					}
					else {
						self.calc_curr_products.push((item_id, 1));
					}
				}
			}
			
			Message::CalcResetThis => {
				self.calc_curr_products.clear();
				self.calc_curr_resources.clear();
			}
			
			_ => {
				println!("JP2 GMD");
			}
		}
    }
	
	fn update_page(&mut self, page: AppPages) {
		match page {
			AppPages::Alchemy => {
				self.calculate_best_alchemy();
			}
			_ => {
				self.last_item = None;
				self.last_item_ge = None;
			}
		}
		self.current_page = page;
		println!("{}", self.current_page.return_current_page_info());
	}
	
	pub fn get_item_by_id (&self, id: usize) -> Option<&osrs::DataHolder> {
		match self.data.iter().find(|thing| thing.id == id) {
			Some(data) => Some(&data),
			None => None,
		}
	}
	
	fn calculate_best_alchemy(&mut self) {
		let options = self.create_filtered_vec(&self.search_filter_alchemy);
		let mut output: Vec<(usize, isize)> = vec![];
		for item in options {
			// volume check
			let volume = match self.item_volume.find(item.id) {
				Some(data) => data,
				None => continue,
			};
			if volume < ALCHEMY_DAILY_VOLUME_LIMIT { continue };
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
			println!("ERROR. No alchemy data");
			return;
		}
		output.sort_by(|a, b| b.1.cmp(&a.1));
		self.best_items_alchemy = output;
	}
	
	pub fn create_filtered_vec(&self, filter: &Option<SearchFilter>) -> Vec<osrs::DataHolder> {
		let mut new_vec = vec![];
		for item in self.data.iter() {
			if item.check_filter(filter) {
				new_vec.push(item.clone());
			}
		}
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
		println!("Selected new item: {}", item.id);
		match self.get_price_from_id(item.id) {
			Ok(data) => {
				self.last_item_ge = Some(data);
				self.last_item = Some(item.clone());
			}
			Err(err) => {
				println!("{err}");
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
		}
		result
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
	app.run()
}