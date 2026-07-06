#![windows_subsystem = "windows"]

use iced::{Element, Center, Size, Pixels, Theme, Subscription, Task};
use iced::widget::{button, column, row, text, space, container, combo_box, stack, center};
use iced::widget::text_editor::{self, Content};
use std::time::{Instant, Duration};

use num_format::{Locale, ToFormattedString};

use reqwest::header::USER_AGENT;
use reqwest::blocking::{Client, Response};

pub mod message;
pub mod osrs;
pub mod structs;
pub mod files;

use message::Message;
use structs::{SearchFilter, AppPages, CurrentRecipe, ItemViewPlot, WindowSizes};
use structs::{ConfigPages, ConfigSettings};

pub const BOND_ID: usize = 13190;
pub const USER_AGENT_MESSAGE: &str = "N3cro0oDev (discord: necro0o) - GE Price Calc Prototype";
pub const APP_SPACING: Pixels = Pixels(5.0);
pub const APP_PADDING: Pixels = Pixels(5.0);
pub const COMBOBOX_MENU_HEIGHT: f32 = 300.0;
pub const ALCHEMY_DAILY_VOLUME_LIMIT: usize = 100;
pub const ALCHEMY_VEC_SIZE: usize = 12;
pub const IMAGE_SIZE_WIDTH: f32 = 128.0;

pub struct MainLayout {
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
	
	pub config_curr_page: ConfigPages,
	pub config_settings: ConfigSettings,
  pub config_window_combo_data: combo_box::State<structs::WindowSizes>,
  pub is_config_changed: bool,
  pub is_new_version: bool,

	pub extra_string: String, // In Calc => recipe label, Alch => max price temp value
	pub extra_string_1: String, // Alch => min price temp value,
	pub extra_string_2: String, // Alch => max volume temp value,
	pub extra_string_3: String, // Alch => min volume temp value,
	pub extra_bool: bool, // In Calc => delete mode, Alch => hide lossy items
	pub extra_bool_1: bool, // In Alch => hide non-members items
	pub extra_bool_2: bool, // In Alch => show only favourites
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
		let vec_item_view = match files::load_view_items () {
			Ok(v) => v,
			Err(err) => {
				log_err!("Cannot get ItemView data. {}", err.to_string());
				vec![]
			}
		};		
		let vec_alch = match files::load_alchemy () {
			Ok(v) => v,
			Err(err) => {
				log_err!("Cannot get Alchemy data. {}", err.to_string());
				vec![]
			}
		};
    let conf_loaded = match files::load_config() {
			Ok(v) => v,
			Err(err) => {
				log_err!("Cannot get Config data. {}", err.to_string());
				ConfigSettings::default()
			}
    };

		let mut layout = MainLayout {
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
			current_page: AppPages::ItemView,
			popup_ready: false,
			
			saved_items_item_view: vec_item_view,
			combo_current_filter_item_view: None,
			selected_item_timeseries_data: None,
			selected_timeseries: osrs::Timeseries::FiveMin,
			
			fav_items_alchemy: vec_alch,
			search_filter_alchemy: Some(SearchFilter::default()),
			best_items_alchemy: vec![],
			table_vec_offset: 0,
			
			calc_curr_recipe: CurrentRecipe::default(),
			calc_saved_recipes: vec,
			calc_description: Content::new(),
			calc_price_multi: 1,
			
			config_curr_page: ConfigPages::AppSettings,
			config_settings: conf_loaded,
		  config_window_combo_data: combo_box::State::new(WindowSizes::all()),
      is_config_changed: false,
      is_new_version: false,

			extra_string: String::new(),
			extra_string_1: String::new(),
			extra_string_2: String::new(),
			extra_string_3: String::new(),
			extra_bool: false,
			extra_bool_1: false,
			extra_bool_2: false,
		};
		let _ = layout.update(Message::RefreshData);
		log_mess!("{:#?}", &layout.calc_saved_recipes);
    let _ = layout.apply_new_settings(false);
		layout
	}
	
	fn title(&self) -> String {
        format!("N3cro0oDev - {}", self.title)
    }
	
	fn subscription(&self) -> Subscription<Message> {
		let update_time = self.config_settings.app_update_interval;
		let tick = iced::time::every(Duration::from_secs(update_time as u64)).map(Message::RefreshTick);
		Subscription::batch(vec![tick, iced::event::listen().map(Message::EventOccurred)])
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

	pub fn update(&mut self, message: Message) -> Task<Message> {
		message::update(self, message)
    }
	
    pub fn apply_new_settings(&mut self, save_config: bool) -> Task<Message> {
        let mut task_to_ret = Task::none();
        if self.current_page == AppPages::Config {
          task_to_ret = match self.config_change_changes() {
            Ok(t) => t,
            Err(err) => {
              log_err!["Error while saving config settings: {}", err];
              Task::none()
            }
          };
        }
        if save_config {
            files::save_config(&self.config_settings).unwrap();
        }
        task_to_ret
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
				self.extra_string = self.config_settings.app_update_interval.to_string();
				self.extra_string_1 = self.config_settings.resolution.0.to_string();
				self.extra_string_2 = self.config_settings.resolution.1.to_string();
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

    pub fn reset_settings(&mut self) {

    }

	fn config_change_changes(&mut self) -> Result<Task<Message>, String> {
		let app_interval = match self.extra_string.parse::<usize>() {
			Ok(i) => {
				if i < 10 { 10 } else { i }
			}
			Err(err) => {
				log_err![err];
				ConfigSettings::default_update_interval() 
			} 
		};
		let width = match self.extra_string_1.parse::<usize>() {
			Ok(i) => {
				if i < 10 { 10 } else { i }
			}
			Err(err) => {
				log_err![err];
				ConfigSettings::default_resolution().0 as usize  
			} 
		};
		let height = match self.extra_string_2.parse::<usize>() {
			Ok(i) => {
				if i < 10 { 10 } else { i }
			}
			Err(err) => {
				log_err![err];
				ConfigSettings::default_resolution().1 as usize  
			} 
		};
		self.config_settings.app_update_interval = app_interval;
		self.config_settings.resolution = (width as f32, height as f32);
		let res = self.config_settings.resolution();
		Ok(iced::window::latest().and_then(move |id| iced::window::resize::<Message>(id, res)))
	}
	
	fn extra_stuff_to_do_once_popup_closes(&mut self) {
		match self.current_page {
			AppPages::Alchemy => {
				self.table_vec_offset = 0;
				self.calculate_best_alchemy();
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
		log_mess!("Alchemy sorting done.");
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
			if item.check_filter(filter, value, volume, &self.fav_items_alchemy) {
				new_vec.push(item.clone());
			}
		}
		log_mess!("Size of new vector: {}", new_vec.len());
		new_vec
	}
	
	fn create_combo_box_data(&mut self) {
		let mut new_vec = self.create_filtered_vec(&self.combo_current_filter_item_view);
    new_vec.push(osrs::DataHolder::bond_holder());
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
    self.check_update();
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
	
pub fn get_alch_fav_vec(&self) -> Vec<String> {
	let mut data_vec: Vec<String> = vec![];
	for data in self.fav_items_alchemy.iter(){
		let diff = match self.best_items_alchemy.iter().find(|item| data.id == item.0) {
			Some(item) => item.1,
			None => 0,
		};
		data_vec.push(format!("{}: {diff} gp", data.name()));
	}
	data_vec
}
	
	fn theme(&self) -> Option<Theme> {
		Some(match self.config_settings.theme {
        Some(t) => { 
            if t >= 0 { Theme::ALL[t as usize].clone() }
            else { /*TODO add custom palette*/ Theme::CatppuccinFrappe}
        }
        None => Theme::CatppuccinFrappe,
    })
	}

  fn check_update(&mut self) {
    let response = self.fetch_get_data("https://github.com/N3cro0o/osrs-ge-helper/releases/latest");
    if let Ok(resp) = response {
        if resp.status().is_success() {
          let body = resp.text().unwrap();
          let o = body.lines()
              .find(|l| l.contains(env!("CARGO_PKG_VERSION")));
          self.is_new_version = o.is_none();
        }
    }
  }
}

impl Default for MainLayout {
	fn default() -> Self {
		MainLayout::new()
	}
}

fn main() -> iced::Result<> {
	unsafe {std::env::set_var("RUST_BACKTRACE", "0");}
	if let Err(err) = files::setup_logger() { return Err(iced::Error::ExecutorCreationFailed(err)) }; // Good enough for now, I believe more Errors should be added to iced::Error 
	log_mess!["INIT APP"];
	
  
  let conf_loaded = match files::load_config() {
    Ok(v) => v,
    Err(err) => {
      log_err!("Cannot get Config data. {}", err.to_string());
      ConfigSettings::default()
    }
  };

  let mut icon_path = std::path::PathBuf::new();
  icon_path.push("img");
  icon_path.push("icon.png");
	let mut window_settings = iced::window::Settings::default();
	window_settings.min_size = Some(Size::new(1280.0,720.0));
	window_settings.size = conf_loaded.resolution();
	window_settings.resizable = false;
  window_settings.icon = iced::window::icon::from_file(icon_path).ok();

	let app = iced::application(MainLayout::default, MainLayout::update, MainLayout::view)
		.window(window_settings)
		.theme(MainLayout::theme)
		.centered()
		.subscription(MainLayout::subscription)
		.title(MainLayout::title);
	let r = app.run();
	log_mess!("APP CLOSE");
	r
}
