use iced::{Task, Event};
use std::sync::LazyLock; // Lazylock to keep Regex in memory
use crate::*;
use crate::structs::{WebPage, WindowSizes};

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
	EventOccurred(Event),
	ResetItemView,
	ResetAlchemy,
	ResetCalculator,
	ResetAllData,
  OpenWebPage(WebPage),
  
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
	AlchemyShowFavourites(bool),
	
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
	CalcClearResource,
	CalcClearProduct,
	
	ConfigChangeUpdateInterval(String),
	ConfigChangeResolutionWidth(String),
	ConfigChangeResolutionHeight(String),
	ConfigChangeResolutionNew(WindowSizes),
  ConfigChangeTheme(Theme),
  OpenExplorer(String),
	
	ChangePlotterTimeseries(osrs::Timeseries),
	ChangeExtraString(String),
	ShowPopup,
	HidePopup,
	ChangeConfigPage(ConfigPages),
}

pub fn update(state: &mut crate::MainLayout, message: Message) -> Task<Message> {
	static RULE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"^[0-9]*$").unwrap());
	match message {
		Message::RefreshData => {
			log_mess!("Get data from OSRS wiki...");
			match state.refresh_data() {
				Ok(size) => log_mess!("Done. Found {} items", size),
				Err(err) => log_err!("{}", err),
			};
			state.bond_sell_price = state.get_price_from_id(BOND_ID).unwrap_or_default().sell_price();
			state.create_combo_box_data();
			state.calculate_best_alchemy();
		}
		
		Message::EventOccurred(_event) => {
			// Add something here later
		}
		
		Message::AddItem(item) => {
			state.select_new_item(&item);
		}
		
		Message::AddItemToSaved => {
			let _ = state.save_current_item();
			if let Err(err) = files::save_view_items(&state.saved_items_item_view) {
				log_err!("{}", err);
			}
		}	

		Message::AlchemyAddToFav(item) => {
			let _ = state.alch_save_current_item(item);
			if let Err(err) = files::save_alchemy(&state.fav_items_alchemy) {
				log_err!("{}", err);
			}
		}
		
		Message::OpenWiki => {
			if let Some(item) = state.last_item.clone() {
				if webbrowser::open(&format!("https://oldschool.runescape.wiki/w/{}", item.name)).is_err() {
					log_mess!("Cannot open wiki");
				}
			}
			else {
				log_err!("No item found");
			}
		}
		
		Message::RemoveItemFromSaved => {
			let _ = state.forget_current_item();
		}
		
		Message::SelectItem(item) => {
			state.select_new_item(&item);
			let _ = state.get_timeseries_data(&item);
		}
		
		Message::ChangePage(page) => {
			return state.update_page(page);
		}
		
		Message::ComboNewFilter(filter) => {
			state.combo_current_filter_item_view = filter;
			state.create_combo_box_data();
		}
		
		Message::AlchNewFilter(filter) => {
			state.search_filter_alchemy = filter;
			state.calculate_best_alchemy();
		}
		
		Message::RefreshTick(_now) => {
			log_mess!("Auto-refresh data from OSRS wiki ...");
			match state.refresh_data() {
				Ok(size) => log_mess!("Done. Found {} items", size),
				Err(err) => log_err!("{}", err),
			};
			state.bond_sell_price = state.get_price_from_id(BOND_ID).unwrap_or_default().sell_price();
			state.create_combo_box_data();
			state.calculate_best_alchemy();
		}
		
		Message::AlchemyDecreaseOffset => {
			if state.table_vec_offset != 0 {
				state.table_vec_offset -= 1;
			}
			let _ = state.update(Message::HidePopup);
		}	
		
		Message::AlchemyIncreaseOffset => {
			if (state.table_vec_offset + 1) * ALCHEMY_VEC_SIZE < state.best_items_alchemy.len() {
				state.table_vec_offset += 1;
			}
			let _ = state.update(Message::HidePopup);
		}
		
		Message::AlchemyChangeMinimumPrice(val) => {
			let minimum = match val.clone().parse::<isize>() {
				Ok(d) => d,
				Err(err) => {
					log_err!("{}", err.to_string());
					return Task::none();
				}
			};
			if minimum > 0 {
				match &mut state.search_filter_alchemy {
					Some(data) => {
						if minimum as usize <= data.maximum_price {
							let _ = data.change_min_price(minimum as usize);
						}
						state.extra_string_1 = val;
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
					return Task::none();
				}
			};
			if maximum > 0 {
				match &mut state.search_filter_alchemy {
					Some(data) => { 
						if maximum as usize >= data.minimum_price {
							let _ = data.change_max_price(maximum as usize); 
						}
						state.extra_string = val;
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
					return Task::none();
				}
			};
			if minimum > 0 {
				match &mut state.search_filter_alchemy {
					Some(data) => { 
						if minimum as usize <= data.maximum_volume {
							let _ = data.change_min_volume(minimum as usize); 
						}
						state.extra_string_3 = val;
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
					return Task::none();
				}
			};
			if maximum > 0 {
				match &mut state.search_filter_alchemy {
					Some(data) => { 
						if data.minimum_volume <= maximum as usize {
							let _ = data.change_max_volume(maximum as usize); 
						}
						state.extra_string_2 = val;
					}
					None => log_mess!("No alchemy filter present. continuing..."),
				}
			}
		}
		
		Message::AlchemyHideLossyItems(b) => {
			state.search_filter_alchemy = match &state.search_filter_alchemy {
				Some(filter) => { 
					state.extra_bool = b;
					Some(filter.clone().change_lossy_items(b))
				}
				None => None,
			};
		}
		
		Message::AlchemyHideMembersItems(b) => {
			state.search_filter_alchemy = match &state.search_filter_alchemy {
				Some(filter) => { 
					state.extra_bool_1 = b;
					Some(filter.clone().change_members_items(b))
				}
				None => None,
			};
		}
		
		Message::AlchemyShowFavourites(b) => {
			state.search_filter_alchemy = match &state.search_filter_alchemy {
				Some(filter) => { 
					state.extra_bool_2 = b;
					Some(filter.clone().change_selected_only(b))
				}
				None => None,
			};
		}
		
		Message::AlchemyCheckItem(item) => {
			let _ = state.update_page(AppPages::ItemView);
			state.select_new_item(&item);
			let _ = state.get_timeseries_data(&item);
		}
		
		Message::CalcAddResource(item_id) => {
			if let Some(_item) = state.get_item_by_id(item_id) {
				if let CurrentRecipe::Loaded(holder) = &mut state.calc_curr_recipe {
					holder.add_one_to_resources(item_id);
				}
				state.recalculate_recipe_prices();
				let _ = state.update(Message::HidePopup);
			}
		}
		Message::CalcAddProduct(item_id) => {
			if let Some(_item) = state.get_item_by_id(item_id) {
				if let CurrentRecipe::Loaded(holder) = &mut state.calc_curr_recipe {
					holder.add_one_to_products(item_id);
				}
				state.recalculate_recipe_prices();
				let _ = state.update(Message::HidePopup);
			}
		}
					
		Message::CalcRemoveResource(item_id) => {
			if let Some(_item) = state.get_item_by_id(item_id) {
				if let CurrentRecipe::Loaded(holder) = &mut state.calc_curr_recipe {
					if let Some(pos) = holder.resources_iter().position(|data_tuple| item_id == data_tuple.id()) { // check if exists
						holder.remove_one_from_resources(pos);
					}
				}
				state.recalculate_recipe_prices();
				let _ = state.update(Message::HidePopup);
			}
		}
		
		Message::CalcRemoveProduct(item_id) => {
			if let Some(_item) = state.get_item_by_id(item_id) {
				if let CurrentRecipe::Loaded(holder) = &mut state.calc_curr_recipe {
					if let Some(pos) = holder.products_iter().position(|data_tuple| item_id == data_tuple.id()) { // check if exists
						holder.remove_one_from_products(pos);
					}
				}
				state.recalculate_recipe_prices();
				let _ = state.update(Message::HidePopup);
			}
		}
		
		Message::CalcClearResource => {
			if let CurrentRecipe::Loaded(holder) = &mut state.calc_curr_recipe {
				holder.clear_resource();
				state.recalculate_recipe_prices();
			}
		}
		
		Message::CalcClearProduct => {
			if let CurrentRecipe::Loaded(holder) = &mut state.calc_curr_recipe {
				holder.clear_product();
				state.recalculate_recipe_prices();
			}
		}
		
		Message::ShowPopup => {
			if !state.popup_ready {
				state.popup_ready = true;
			} // Now every button works as a toggle
			else {
				state.popup_ready = false;
				state.extra_stuff_to_do_once_popup_closes();
			}
		}
		
		Message::HidePopup => {
			if state.popup_ready {
				state.popup_ready = false;
				state.extra_stuff_to_do_once_popup_closes();
			}
		}
		
		Message::ChangePlotterTimeseries(timeseries) => {
			state.selected_timeseries = timeseries;
			if let Some(item) = &state.last_item {
				let _ = state.get_timeseries_data(&item.clone());
			}
		}
		
		Message::CalcResetThis => {
			state.calc_curr_recipe = CurrentRecipe::new();
			state.calc_description = Content::new();
			let _ = state.update(Message::HidePopup);
		}
		
		Message::CalcChangePriceMultiplier(multi_str) => {
			let multi = match multi_str.parse::<isize>() {
				Ok(d) => d,
				Err(err) => {
					log_err!("{}", err.to_string());
					return Task::none();
				}
			};
			if multi > 0 {
				state.calc_price_multi = multi as usize;
				state.recalculate_recipe_prices();
			}
		}
		
		Message::CurrentTesat => {
			state._debug_value = !state._debug_value;
		}
		
		Message::CalcAcceptRecipeName => {
			if let CurrentRecipe::Loaded(holder) = &mut state.calc_curr_recipe {
				holder.set_id(state.calc_saved_recipes.len()).set_label(state.extra_string.clone())
					.set_desc(state.calc_description.text());
				if let Err(err) = files::save_recipe(&holder) {
					log_err!("{}", err.to_string());
					return Task::none();
				}
				state.popup_ready = false;
				for data in state.calc_saved_recipes.iter() {
					let str_offset = data.find(' ').unwrap_or(0);
					let val = data[..str_offset].to_string().parse::<usize>().unwrap_or_default();
					if holder.id == val {
						return Task::none();
					}
				}
				state.calc_saved_recipes.push(format!("{} {}", holder.id, holder.label.clone()));
			}
		}
		
		Message::CalcSelectItem(id) => {
			let data = match files::load_recipe(id) {
				Ok(d) => d,
				Err(err) => {
					log_err!("{}", err);
					return Task::none();
				}
			};
			state.extra_string = data.label.clone();
			state.calc_description = Content::with_text(&data.description);
			state.calc_curr_recipe = CurrentRecipe::from(data);
			state.recalculate_recipe_prices();
		}
		
		Message::CalcDeleteItem(id) => {
			if let Err(err) = files::delete_recipe(id) {
				log_err!("{}", err);
				return Task::none();
			}
			let _ = state.update(Message::CalcDisableDelMode);
			for i in 0..state.calc_saved_recipes.len() {
				let data = &state.calc_saved_recipes[i];
				let str_offset = data.find(' ').unwrap_or(0);
				let val = data[..str_offset].to_string().parse::<usize>().unwrap_or_default();
				if id == val {
					state.calc_saved_recipes.remove(i);
					break;
				}
			}
			state.calc_description = Content::new();
		}
				
		Message::CalcChangeItemDesc(action) => {
			if let CurrentRecipe::Loaded(_) = state.calc_curr_recipe {
				state.calc_description.perform(action)
			}
		}
		
		Message::ChangeConfigPage(page) => {
			state.config_curr_page = page;
		}

		Message::ConfigChangeUpdateInterval(time_str) => {
			if RULE.is_match(&time_str) {
				state.extra_string = time_str;
			}
		}
		
		Message::ConfigChangeResolutionWidth(x) => {
			if RULE.is_match(&x) { state.extra_string_1 = x; }
		}	
		
		Message::ConfigChangeResolutionHeight(y) => {
			if RULE.is_match(&y) { state.extra_string_2 = y; }
		}

    Message::ConfigChangeResolutionNew(res) => {
        state.config_settings.new_resolution = res;
    }

    Message::ConfigChangeTheme(theme) => {
        log_mess!["{:?}", theme];
    }

		Message::OpenExplorer(path) => {
			log_mess!["Openning file explorer in {}", &path];
			if let Err(err) = open::that(path) {
				log_err!["Error while openning path: {}", err];
			}
		}
		
		Message::CalcEnableDelMode => {
			state.extra_bool = true;
		}
		
		Message::CalcDisableDelMode => {
			state.extra_bool = false;
		}
		
		Message::ChangeExtraString(string) => {
			state.extra_string = string;
		}
		
		Message::ResetItemView => {
			log_mess!["Deleting ItemView data..."];
			if let Err(err) = files::delete_item_view() {
				log_err!["Error while deleting data: {}", err];
				return Task::none();
			}
			state.saved_items_item_view.clear();
			log_mess!["DONE"];
		}
				
		Message::ResetAlchemy => {
			log_mess!["Deleting Alchemy data..."];
			if let Err(err) = files::delete_alchemy() {
				log_err!["Error while deleting data: {}", err];
				return Task::none();
			}
			state.fav_items_alchemy.clear();
			log_mess!["DONE"];
		}
				
		Message::ResetCalculator => {
			log_mess!["Deleting Calculator data..."];
			if let Err(err) = files::delete_all_recipes() {
				log_err!["Error while deleting data: {}", err];
				return Task::none();
			}
			state.calc_saved_recipes.clear();
			state.calc_curr_recipe = CurrentRecipe::Empty;
			state.calc_description = Content::new();
			log_mess!["DONE"];
		}
				
		Message::ResetAllData => {
			log_mess!["Deleting all data..."];
			log_mess!["Deleting ItemView data..."];
			if let Err(err) = files::delete_item_view() {
				log_err!["Error while deleting data: {}", err];
				return Task::none();
			}
			log_mess!["Deleting Alchemy data..."];
			if let Err(err) = files::delete_alchemy() {
				log_err!["Error while deleting data: {}", err];
				return Task::none();
			}
			log_mess!["Deleting Calculator data..."];
			if let Err(err) = files::delete_all_recipes() {
				log_err!["Error while deleting data: {}", err];
				return Task::none();
			}
			state.saved_items_item_view.clear();
			state.fav_items_alchemy.clear();
			state.calc_saved_recipes.clear();
			state.calc_curr_recipe = CurrentRecipe::Empty;
			state.calc_description = Content::new();
			log_mess!["DONE"];
		}
	
    Message::OpenWebPage(page) => {
      if webbrowser::open(page.get_url()).is_err() {
        log_mess!("Cannot open web page");
      }
    }

		_ => {
			log_mess!("Invalid Message detected");
			return iced::window::latest().and_then(iced::window::close);
		}
	}
	Task::none()
}
