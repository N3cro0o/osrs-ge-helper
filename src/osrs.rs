use std::fmt;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::slice::Iter;
use super::structs;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DataHolder {
	pub name: String,
	icon: String,
	pub examine: String,
	pub id: usize,
	members: bool,
	lowalch: Option<usize>,
	limit: Option<u32>,
	value: Option<usize>,
	highalch: Option<usize>,
  #[serde(skip)]
  pub price_threshold: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DataThresholdHolder {
    pub id: usize,
    pub price_threshold: Option<usize>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct GEData {
	high: Option<usize>,
	#[serde(rename = "highTime")]
	high_time: Option<usize>,
	low: Option<usize>,
	#[serde(rename = "lowTime")]
	low_time: Option<usize>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct VolumeData {
	timestamp: usize,
	data: HashMap<String, usize>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct LatestData {
	data: HashMap<String, GEData>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Timeseries {
	FiveMin,
	OneHour,
	SixHour,
	TwentyFourHours,
}

impl Timeseries {
	#[allow(non_snake_case)]
	pub const fn ALL() -> [Self; 4] {
		[Self::FiveMin, Self::OneHour, Self::SixHour, Self::TwentyFourHours]
	}
}

impl std::fmt::Display for Timeseries {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let string = match self {
			Timeseries::FiveMin => String::from("5m"),
			Timeseries::OneHour => String::from("1h"),
			Timeseries::SixHour => String::from("6h"),
			Timeseries::TwentyFourHours => String::from("24h"),
		};
		write!(f, "{}", string)
	}
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct TimeseriesData {
	data: Vec<TimeseriesItemData>,
	#[serde(rename = "itemId")]
	item_id: usize,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct TimeseriesItemData {
	pub timestamp: usize,
	#[serde(rename = "avgHighPrice")]
	high_price_average: Option<usize>,
	#[serde(rename = "avgLowPrice")]
	low_price_average: Option<usize>,
	#[serde(rename = "highPriceVolume")]
	high_price_volume: Option<usize>,
	#[serde(rename = "lowPriceVolume")]
	low_price_volume: Option<usize>,
}

impl DataHolder {
  pub fn bond_holder() -> Self {
      DataHolder{
          name: "Old school bond".to_string(),
          icon: "".to_string(),
          examine: "This bond can be redeemed for membership.".to_string(),
          id: 13190,
          members: false,
          lowalch: None,
          limit: None,
          value: None,
          highalch: None,
          price_threshold: None,
      }
  }

	pub fn name(&self) -> String {
		self.name.clone()
	}
	
	pub fn short_description(&self) -> String {
		format!("({}) {}", self.id, self.name)
	}
	
	pub fn basic_data(&self) -> (usize, usize, usize) {
		let tuple = (
			self.value.unwrap_or(0),
			self.lowalch.unwrap_or(0),
			self.highalch.unwrap_or(0)
		);
		tuple
	}

  pub fn value(&self) -> Option<usize> {
      self.value.clone()
  }
	
	pub fn check_filter(&self, filter: &Option<structs::SearchFilter>, price: usize, volume: usize, list: &Vec<Self>) -> bool {
		if let Some(f) = filter {
			if f.only_non_member_items && self.members {
				return false;
			}
			if price < f.minimum_price || price > f.maximum_price {
				return false;
			}
			if f.hide_loss_alch && (self.basic_data().2 as isize - price as isize) < 0 {
				return false;
			}
			if volume < f.minimum_volume || volume > f.maximum_volume {
				return false;
			}
			if f.only_selected {
				if let None = list.iter().position(|item| self == item) {
					return false;
				}
			}
			true
		}
		else {
			true
		}
	}
}

impl std::convert::Into<DataThresholdHolder> for DataHolder {
    fn into(self) -> DataThresholdHolder {
        DataThresholdHolder{
            id: self.id,
            price_threshold: self.price_threshold,
        }
    }
}

impl std::convert::Into<DataThresholdHolder> for &DataHolder {
    fn into(self) -> DataThresholdHolder {
        DataThresholdHolder{
            id: self.id,
            price_threshold: self.price_threshold,
        }
    }
}

impl fmt::Display for DataHolder {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.short_description())
	}
}

impl std::cmp::PartialEq for DataHolder {
    fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}

    fn ne(&self, other: &Self) -> bool {
		self.id != other.id
	}
}

impl GEData {
	pub fn basic_data(&self) -> (usize, usize) {
		let tuple = (
			self.high.unwrap_or(0),
			self.low.unwrap_or(0),
		);
		tuple
	}
	
	pub fn sell_price(&self) -> Option<usize> {
		self.high.clone()
	}
	
	pub fn buy_price(&self) -> Option<usize> {
		self.low.clone()
	}
}

impl fmt::Display for GEData {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let high = self.high.unwrap_or_default();
		let low = self.low.unwrap_or_default();
		let high_time = self.high_time.unwrap_or_default();
		let low_time = self.low_time.unwrap_or_default();
		write!(f, "High price: {} ({}), low price: {}({})", high, high_time, low, low_time)
	}
}

impl VolumeData {
	pub fn find(&self, id: usize) -> Option<usize> {
		let id_str: String = id.to_string();
		self.data.get(&id_str).copied()
	}
}

impl LatestData {
	pub fn get_data_by_id(&self, id: usize) -> Option<GEData> {
		self.data.get(&id.to_string()).copied()
	}
}

impl TimeseriesData {
	pub fn get_data_iter(&self) -> Iter<'_, TimeseriesItemData> {
		self.data.iter()
	}
	
	pub fn get_time_tuple(&self) -> (usize, usize) {
		let first = self.data[0].timestamp;
		let last = self.data[364].timestamp;
		(first, last)
	}
}

impl TimeseriesItemData {
	pub fn high_price_average(&self) -> Option<usize> {
		self.high_price_average.clone()
	}
	pub fn low_price_average(&self) -> Option<usize> {
		self.low_price_average.clone()
	}
	pub fn high_price_volume(&self) -> Option<usize> {
		self.high_price_volume.clone()
	}
	pub fn low_price_volume(&self) -> Option<usize> {
		self.low_price_volume.clone()
	}
}
