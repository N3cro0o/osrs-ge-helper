// TODO: Refactor AppPages implementations to different files

use chrono::{TimeZone, Local};
use serde::{Serialize, Deserialize};

use iced::{Element, Length};
use iced::widget::{text, center, container};

use plotters::{coord::Shift, prelude::*};
use plotters_backend::DrawingBackend;
use plotters_iced2 as plotters_iced;
use plotters_iced::{plotters_backend, Chart, ChartWidget, DrawingArea};

use crate::{Message, MainLayout};
use crate::osrs;

mod item_view;
mod alchemy;
mod calculator;
mod config;

pub const SKIP_VARIABLE: u16 = 0;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchFilter {
	pub only_non_member_items: bool
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum AppPages {
	#[default]
	ItemView,
	Alchemy,
	Calculator,
	Config,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecipeElement (usize, usize);

#[derive(Debug)]
pub enum RecipePages {
	CalculatorPage,
	NotesPage,
	ProfitPage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecipeHolder {
	calc_curr_resources: Vec<RecipeElement>,
	#[serde(default)]
	calc_curr_resources_extra: Vec<usize>,
	calc_curr_products: Vec<RecipeElement>,
	#[serde(default)]
	calc_curr_products_extra: Vec<usize>,
	pub description: String, // TODO: change for text_editor content
	pub label: String,
	pub id: usize,
	#[serde(skip)]
	pub page: RecipePages,
	#[serde(skip)]
	pub resc_cost: isize,
	#[serde(skip)]
	pub prod_cost: isize,
	#[serde(skip)]
	pub reci_cost: isize,
}

pub enum CurrentRecipe {
	Loaded(RecipeHolder),
	Empty,
}

#[derive(Default)]
pub struct ItemViewPlot {
	item_name: String,
	data_series: Option<osrs::TimeseriesData>,
}

impl ItemViewPlot {
	pub fn view(&self) -> Element<'_, Message> {
		let chart = ChartWidget::new(self)
			.width(Length::Fill)
			.height(Length::Fill);
		
		chart.into()
	}
	
	pub fn change_label(&mut self, item_name: String) {
		self.item_name = item_name;
	}
	
	pub fn update_data(&mut self, data: osrs::TimeseriesData) {
		self.data_series = Some(data);
	}
	
	pub fn reset_data(&mut self) {
		self.data_series = None;
		self.item_name = String::new();
	}
}

impl Chart<Message> for ItemViewPlot {
    type State = ();
    // leave it empty
    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, _builder: ChartBuilder<DB>) {}

    fn draw_chart<DB: DrawingBackend>(&self, _state: &Self::State, root: DrawingArea<DB, Shift>) {
		let y_margin: (usize, usize) = {
			if let Some(data) = &self.data_series {
				let mut min: usize = usize::MAX;
				let mut max: usize = 0;

				for item in data.get_data_iter() {
					if let Some(high) = item.high_price_average() {
						if min > high {
							min = high;
						}
						else if max < high {
							max = high;
						}
					}
				}
				
				((min as f32 * 0.995) as usize, (max as f32 * 1.005) as usize)
			}
			else {
				(0, 2)
			}
		};
		let x_margin: (usize, usize) = {
			if let Some(data) = &self.data_series {
				data.get_time_tuple()
			}
			else {
				(0, 3)
			}
		};
		let data: Vec<(usize, usize)> = {
			if let Some(data) = &self.data_series {
				let mut vec: Vec<(usize, usize)> = vec![];
				let mut skip_index = SKIP_VARIABLE;
				for item in data.get_data_iter() {
					if skip_index < SKIP_VARIABLE {
						skip_index += 1;
						continue;
					}
					else { skip_index = 0; }
					if let Some(high) = &item.high_price_average() {
						vec.push((item.timestamp, *high));
					}
					else {
						let x = match vec.last() {
							Some((_x, y)) => (item.timestamp, *y),
							None => (item.timestamp, 0)
						};
						vec.push(x);
					}
				}
				vec
			}
			
			else {
				vec![(0, 1), (1, 1), (2, 1), (3, 1)]
			}
		};
		
        let mut builder = ChartBuilder::on(&root);
		let mut chart = builder
			.margin(30)
			.x_label_area_size(30)
			.y_label_area_size(30)
			.build_cartesian_2d((x_margin.0)..(x_margin.1), (y_margin.0)..(y_margin.1))
			.unwrap();

		chart
			.configure_mesh()
			.x_labels(7)
			.x_label_formatter(&|x| {
				let local_time = Local.timestamp_opt(*x as i64, 0).single();
				match local_time {
					Some(time) => format!("{}", time.format("%d.%m.%Y %H:%M")),
					None => x.to_string(),
				}
			})
			.y_labels(5)
			.draw()
			.unwrap();

		chart
			.draw_series(LineSeries::new(
				data,
				&RED,
			))
			.unwrap();
    }
}

impl SearchFilter {
	pub fn new() -> Self {
		SearchFilter {
			only_non_member_items: false
		}
	}
	
	pub fn flip_member_items(&mut self) -> Self {
		self.only_non_member_items = !self.only_non_member_items;
		self.clone()
	}
}

impl Default for SearchFilter {
	fn default() -> Self {
		SearchFilter::new()
	}
}

impl AppPages {
	pub fn return_current_page_info(&self) -> String {
		match self {
			AppPages::ItemView => format!("Current Page -> Item view"),
			AppPages::Alchemy => format!("Current Page -> Alchemy view"),
			AppPages::Calculator => format!("Current Page -> Recipe calculator"),
			AppPages::Config => format!("Current Page -> Config"),
		}
	}
	
	pub fn sidebar<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		match self {
			AppPages::ItemView => self.item_sidebar_view(state),
			AppPages::Alchemy => self.alch_sidebar_view(state),
			AppPages::Calculator => self.calc_sidebar_view(state),
			AppPages::Config => self.config_sidebar_view(state),
		}
	}
	
	pub fn body<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		match self {
			AppPages::ItemView => self.item_body_view(state),
			AppPages::Alchemy => self.alch_body_view(state),
			AppPages::Calculator => self.calc_body_view(state),
			AppPages::Config => self.config_body_view(state),
		}
	}
	
	pub fn overlay<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		match self {
			AppPages::Calculator => self.calc_overlay_view(state),
			_ => self._other_overlay_view(),
		}
	}
	
	fn _other_sidebar_view<'a>(&'a self) -> Element<'a, Message> {
		center(text("Nothing")).into()
	}
	
	fn _other_body_view<'a>(&'a self) -> Element<'a, Message> {
		center(text("Nothing")).into()
	}
	
	fn _other_overlay_view<'a>(&'a self) -> Element<'a, Message> {
		center(text("No overlay m8"))
			.style(container::rounded_box)
			.height(Length::Fixed(200.0))
			.width(Length::Fixed(400.0))
			.into()
	}
}

impl CurrentRecipe {
	pub fn new() -> Self {
		Self::Loaded(RecipeHolder::default())
	}
	
	pub fn from(data: RecipeHolder) -> Self{
		Self::Loaded(data)
	}
}

impl Default for CurrentRecipe {
	fn default() -> Self {
		CurrentRecipe::Empty
	}
}

impl RecipeHolder {
	pub fn is_products_empty(&self) -> bool {
		self.calc_curr_products.is_empty()
	}
	
	pub fn is_resources_empty(&self) -> bool {
		self.calc_curr_resources.is_empty()
	}
	
	pub fn is_products_empty_extra(&self) -> bool {
		self.calc_curr_products_extra.is_empty()
	}
	
	pub fn is_resources_empty_extra(&self) -> bool {
		self.calc_curr_resources_extra.is_empty()
	}
	
	/// Update ID only if ID field is empty (ID == usize::MAX)
	pub fn set_id(&mut self, new_id: usize) -> &mut Self {
		if self.id == usize::MAX {
			self.id = new_id;
		}
		self
	}	
	
	pub fn set_label(&mut self, new_label: String) -> &mut Self {
		self.label = new_label;
		self
	}
	
	pub fn set_desc(&mut self, new_description: String) -> &mut Self {
		self.description = new_description;
		self
	}
	
	pub fn resources_iter(&self) -> std::slice::Iter<'_, RecipeElement> {
		self.calc_curr_resources.iter()
	}
	
	pub fn products_iter(&self) -> std::slice::Iter<'_, RecipeElement> {
		self.calc_curr_products.iter()
	}
	
	pub fn add_one_to_resources(&mut self, id: usize) {
		if let Some(pos) = self.calc_curr_resources.iter().position(|data_tuple| id == data_tuple.0) {
			self.calc_curr_resources[pos].1 += 1;
		}
		else {
			self.calc_curr_resources.push(RecipeElement(id, 1));
		}
	}	
	
	pub fn add_one_to_products(&mut self, id: usize) {
		if let Some(pos) = self.calc_curr_products.iter().position(|data_tuple| id == data_tuple.0) {
			self.calc_curr_products[pos].1 += 1;
		}
		else {
			self.calc_curr_products.push(RecipeElement(id, 1));
		}
	}
	
	pub fn remove_one_from_products(&mut self, pos: usize) {
		if self.calc_curr_products[pos].1 > 1 {
			self.calc_curr_products[pos].1 -= 1;
		}
		else {
			self.calc_curr_products.remove(pos);
		}
	}
	
	pub fn remove_one_from_resources(&mut self, pos: usize) {
		if self.calc_curr_resources[pos].1 > 1 {
			self.calc_curr_resources[pos].1 -= 1;
		}
		else {
			self.calc_curr_resources.remove(pos);
		}
	}
}

impl Default for RecipeHolder {
	fn default() -> Self {
		RecipeHolder {
			calc_curr_resources: vec![],
			calc_curr_resources_extra: vec![],
			calc_curr_products: vec![],
			calc_curr_products_extra: vec![],
			description: String::new(),
			label: String::from("New recipe"),
			id: usize::MAX,
			page: RecipePages::CalculatorPage,
			resc_cost: 0,
			prod_cost: 0,
			reci_cost: 0,
		}
	}
}

impl RecipeElement {
	pub fn id(&self) -> usize {
		self.0
	}
	
	pub fn num(&self) -> usize {
		self.1
	}
}

impl Default for RecipePages {
	fn default() -> Self {
		Self::CalculatorPage
	}
}