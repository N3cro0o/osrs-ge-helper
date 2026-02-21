use std::fmt;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
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
	highalch: Option<usize>
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GEData {
	high: Option<usize>,
	high_time: Option<usize>,
	low: Option<usize>,
	low_time: Option<usize>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct VolumeData {
	timestamp: usize,
	data: HashMap<String, usize>,
}

impl DataHolder {
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
	
	pub fn check_filter(&self, filter: &Option<structs::SearchFilter>) -> bool {
		if let Some(f) = filter {
			if f.only_non_member_items && self.members {
				return false;
			}
			true
		}
		else {
			true
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
}

impl VolumeData {
	pub fn find(&self, id: usize) -> Option<usize> {
		let id_str: String = id.to_string();
		self.data.get(&id_str).copied()
	}
}