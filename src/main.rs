use iced::{Element, Center, Size, Pixels, Theme};
use iced::widget::{button, column, row, text, space, container, combo_box};

use num_format::{Locale, ToFormattedString};

use reqwest::header::USER_AGENT;
use reqwest::blocking::{Client, Response};

mod osrs;
mod structs;

use structs::{SearchFilter, AppPages};

pub const APP_VERSION: &str = "0.1.";
pub const BOND_ID: usize = 13190;
pub const USER_AGENT_MESSAGE: &str = "N3cro0oDev (necro0o) - GE Price Calc Prototype";
pub const APP_SPACING: Pixels = Pixels(5.0);
pub const APP_PADDING: Pixels = Pixels(5.0);
pub const COMBOBOX_MENU_HEIGHT: f32 = 300.0;

#[derive(Default)]
pub struct MainLayout {
    pub _debug_value: bool,
	pub data: Vec<osrs::DataHolder>,
	pub combo_data: combo_box::State<osrs::DataHolder>,
	pub item_volume: osrs::VolumeData,
	pub bond_sell_price: Option<usize>,
	pub picked_items: Vec<osrs::DataHolder>,
	pub last_item: Option<osrs::DataHolder>,
	pub last_item_ge: Option<osrs::GEData>,
	pub title: String,
	pub theme: Option<Theme>,
	pub current_page: AppPages,
	pub combo_current_filter: Option<SearchFilter>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
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
}

impl MainLayout {
	pub fn default() -> Self {
		let vec: Vec<osrs::DataHolder> = vec![];
		let theme = Some(Theme::CatppuccinFrappe);
		let mut layout = MainLayout {
			_debug_value: false,
			data: vec![],
			combo_data: combo_box::State::new(vec),
			item_volume: osrs::VolumeData::default(),
			bond_sell_price: None,
			last_item: None,
			last_item_ge: None,
			picked_items: vec![],
			title: "OSRS GE Calculator".to_string(),
			theme,
			current_page: AppPages::ItemView,
			combo_current_filter: None,
		};
		layout.update(Message::RefreshData);
		layout
	}
	
	fn title(&self) -> String {
        format!("N3cro0oDev - {}", self.title)
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
							// .on_press_maybe(
								// (self.current_page != AppPages::Alchemy)
									// .then_some(Message::ChangePage(AppPages::Alchemy))
								// )
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
							// .on_press_maybe(
								// (self.current_page != AppPages::Calculator)
									// .then_some(Message::ChangePage(AppPages::Calculator))
								// )
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
				self.bond_sell_price = self.get_price_from_id(BOND_ID).sell_price();
				self._debug_value = !self._debug_value;
				self.create_combo_box_data();
			}
			
			Message::AddItem(item) => {
				self.select_new_item(&item);
			}
			
			Message::AddItemToSaved => {
				let _ = self.save_current_item();
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
				self.current_page = page;
				println!("{}", self.current_page.return_current_page_info());
			}
			
			Message::ComboNewFilter(filter) => {
				self.combo_current_filter = filter;
				self.create_combo_box_data();
			}
			
			_ => {
				println!("JP2 GMD");
			}
		}
    }
	
	fn create_combo_box_data(&mut self) {
		let mut new_vec = vec![];
		for item in self.data.iter() {
			if item.check_filter(&self.combo_current_filter) {
				new_vec.push(item.clone());
			}
		}
		self.combo_data = combo_box::State::new(new_vec);
	}
	
	fn save_current_item(&mut self) -> Result<(), (u8, String)> {
		if let None = self.last_item {
			return Err((1, String::from("No selected item")));
		}
		self.picked_items.push(self.last_item.clone().unwrap());
		Ok(())
	}
	
	fn forget_current_item(&mut self) -> Result<(), (u8, String)> {
		if let None = self.last_item {
			return Err((1, String::from("No selected item")));
		}
		let last_item = self.last_item.clone().unwrap(); 
		if let Some(pos) = self.picked_items.iter().position(|vec_item| vec_item.id == last_item.id) {
			let _ = self.picked_items.remove(pos);
		}
		Ok(())
	}
	
	fn select_new_item(&mut self, item: &osrs::DataHolder){
		println!("Selected new item: {}", item.id);
		let data = self.get_price_from_id(item.id);
		self.last_item_ge = Some(data);
		self.last_item = Some(item.clone());
	}
	
	fn get_price_from_id(&self, id: usize) -> osrs::GEData {
		let client = Client::new();
		let response = client.get(format!("https://prices.runescape.wiki/api/v1/osrs/latest?id={}", id))
			.header(USER_AGENT, USER_AGENT_MESSAGE)
			.send()
			.unwrap();
		let body = response.text().unwrap();
		let index = body.find(&id.to_string()).unwrap();
		let body = &body[index + &id.to_string().len() + 2 .. body.len() - 2];
		let data = serde_json::from_str::<osrs::GEData>(&body).unwrap();
		data
	}
	
	fn refresh_data(&mut self) -> Result<usize, String> {
		let result = self.refresh_item_data();
		if let Ok(_) = result {
			self.refresh_volume_data()?;
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
	
	fn fetch_get_data(&mut self, url: &str) -> reqwest::Result<Response> {
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

fn main() -> iced::Result<> {
	let mut window_settings = iced::window::Settings::default();
	window_settings.min_size = Some(Size::new(1280.0,720.0));
	window_settings.size = Size::new(1280.0,720.0);
	window_settings.resizable = true;
	
	let app = iced::application(MainLayout::default, MainLayout::update, MainLayout::view)
		.window(window_settings)
		.theme(MainLayout::theme)
		.centered()
		.title(MainLayout::title);
	app.run()
}