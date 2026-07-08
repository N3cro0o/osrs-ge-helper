use std::io::{self, Error, ErrorKind, Write, Read};
use std::{path, fs};
use serde::{Serialize, Deserialize};

use crate::structs::{RecipeHolder, ConfigSettings};
use crate::osrs;
use crate::log_mess;

use super::{check_dir, create_dir};
use super::{DATA_DIR, RECIPES_DIR, FAVDATA_DIR, ITEM_FILE, THRES_FILE, ALCH_FILE, CONFIG_DIR, CONFIG_FILE};

#[derive(Serialize, Deserialize)]
struct Wrapper<T: std::clone::Clone> {
	data: Vec<T>,
}

impl<T: std::clone::Clone> Wrapper<T> {
	pub fn from(data: &Vec<T>) -> Self {
		Wrapper {
			data: data.to_vec()
		}
	}
	
	pub fn return_data(&self) -> Vec<T> {
		self.data.clone()
	}
}

pub fn save_recipe(data: &RecipeHolder) -> io::Result<()> {
	let mut path = get_local_data_dir()?;
	path.push(RECIPES_DIR);
	if !check_dir(&path) {
		create_dir(&path)?;
	}
	path.push(format!("{}-{}.xml", data.id, data.label));
	dbg!(&path);
	let mut file = fs::OpenOptions::new().write(true).truncate(true).create(true).open(&path)?;
	let xml_data = match serde_xml_rs::to_string(data) {
			Ok(string) => string,
			Err(err) => return Err(Error::new(ErrorKind::Other, err)),
	};
	file.write_all(xml_data.as_bytes())?;
	Ok(())
}

pub fn save_view_items(data: &Vec<osrs::DataHolder>) -> io::Result<()> {
  save_view_items_thresholds(data)?;
  log_mess!["Saving main ItemView data..."];
	let mut path = get_local_data_dir()?;
	path.push(FAVDATA_DIR);
	if !check_dir(&path) {
		create_dir(&path)?;
	}
	path.push(ITEM_FILE);
	let file = fs::OpenOptions::new().write(true).truncate(true).create(true).open(&path)?;
	let mut stream = io::BufWriter::new(file);
	let wrapper = Wrapper::from(data);
	let xml_data = match serde_xml_rs::to_string(&wrapper) {
		Ok(string) => string,
		Err(err) => return Err(Error::new(ErrorKind::Other, err)),
	};
	stream.write(xml_data.as_bytes())?;
  log_mess!["Done"];
	Ok(())
}

fn save_view_items_thresholds(data: &Vec<osrs::DataHolder>) -> io::Result<()> {
    log_mess!["Saving ItemView threshold data..."];
    let new_vec: Vec<osrs::DataThresholdHolder> = data.iter().map(|item| item.into()).collect();
    let mut path = get_local_data_dir()?;
    path.push(FAVDATA_DIR);
    if !check_dir(&path) {
        create_dir(&path)?;
    }
    path.push(THRES_FILE);
    let file = fs::OpenOptions::new().write(true).truncate(true).create(true).open(&path)?;
    let mut stream = io::BufWriter::new(file);
    let wrapper = Wrapper::from(&new_vec);
    let xml_data = match serde_xml_rs::to_string(&wrapper) {
        Ok(string) => string,
        Err(err) => return Err(Error::new(ErrorKind::Other, err)),
    };
    stream.write(xml_data.as_bytes())?;
    Ok(())
}

pub fn save_alchemy(data: &Vec<osrs::DataHolder>) -> io::Result<()> {
	let mut path = get_local_data_dir()?;
	path.push(FAVDATA_DIR);
	if !check_dir(&path) {
		create_dir(&path)?;
	}
	path.push(ALCH_FILE);
	let file = fs::OpenOptions::new().write(true).truncate(true).create(true).open(&path)?;
	let mut stream = io::BufWriter::new(file);
	let wrapper = Wrapper::from(data);
	let xml_data = match serde_xml_rs::to_string(&wrapper) {
		Ok(string) => string,
		Err(err) => return Err(Error::new(ErrorKind::Other, err)),
	};
	stream.write(xml_data.as_bytes())?;
	Ok(())
}

pub fn save_config(data: &ConfigSettings) -> io::Result<()>{
	let mut path = get_local_data_dir()?;
	path.push(CONFIG_DIR);
	if !check_dir(&path) {
		create_dir(&path)?;
	}
	path.push(CONFIG_FILE);
	let file = fs::OpenOptions::new().write(true).truncate(true).create(true).open(&path)?;
	let mut stream = io::BufWriter::new(file);
	let xml_data = match serde_xml_rs::to_string(data) {
		Ok(string) => string,
		Err(err) => return Err(Error::new(ErrorKind::Other, err)),
	};
	stream.write(xml_data.as_bytes())?;
  crate::log_mess!["Config file saved!"];
	Ok(())
}

pub fn load_recipes_vec() -> io::Result<Vec<String>> {
	let mut vec = vec![];
	let mut path = get_local_data_dir()?;
	path.push(RECIPES_DIR);
	if !check_dir(&path) {
		return Err(Error::new(ErrorKind::Other, "Recipes path doesn't exist"));
	}
	for entry in fs::read_dir(&path)? {
		let entry = entry?;
		let val = match entry.file_name().into_string() {
			Ok(mut string) => {
				string = string.replace('-', " ");
				let str_offset = string.find(".xml").unwrap_or(string.len());
				string[..str_offset].to_string()
			}
			Err(_) => {
				crate::log_err!("Cannot parse OsStrint to String... continuing");
				continue;
			}
		};
		vec.push(val);
	}
	Ok(vec)
}

pub fn load_view_items() -> io::Result<Vec<osrs::DataHolder>> {
  let mut thresh_vec = load_view_items_threshold()?;
  thresh_vec.sort_by_key(|item| item.id);
	let mut path = get_local_data_dir()?;
	path.push(FAVDATA_DIR);
	if !check_dir(&path) {
		return Err(Error::new(ErrorKind::Other, "ItemView saved items path doesn't exist"));
	}
	path.push(ITEM_FILE);
	let mut file = fs::OpenOptions::new().read(true).open(&path)?;
	let mut wrapper_str = String::new();
	let _ = file.read_to_string(&mut wrapper_str)?;
	let wrapper: Wrapper<osrs::DataHolder> = match serde_xml_rs::from_str(&wrapper_str) {
		Ok(d) => d,
		Err(err) => return Err(Error::new(ErrorKind::Other, format!("Cannot deserialize Wrapper struct for ItemView data. {err}"))),
	};
  let mut v = wrapper.return_data();
  v.sort_by_key(|item| item.id);
  merge_threshold(&mut v, thresh_vec);
	Ok(v)
}

fn merge_threshold(vec: &mut Vec<osrs::DataHolder>, thresh: Vec<osrs::DataThresholdHolder>) {
    let mut i = 0;
    let mut j = 0;
    log_mess!["Start ItemView threshold merge..."];
    while i < vec.len() && j < thresh.len() {
        if vec[i].id == thresh[j].id {
            vec[i].price_threshold = thresh[j].price_threshold;
            i = i + 1;
        }
        else if vec[i].id < thresh[j].id {
            i = i + 1;
        }
        else {
            j = j + 1;
        }
    }
    log_mess!["Done"]
}

fn load_view_items_threshold() -> io::Result<Vec<osrs::DataThresholdHolder>> {
	let mut path = get_local_data_dir()?;
	path.push(FAVDATA_DIR);
	if !check_dir(&path) {
		return Err(Error::new(ErrorKind::Other, "ItemView saved items path doesn't exist"));
	}
	path.push(THRES_FILE);
	let mut file = fs::OpenOptions::new().read(true).open(&path)?;
	let mut wrapper_str = String::new();
	let _ = file.read_to_string(&mut wrapper_str)?;
	let wrapper: Wrapper<osrs::DataThresholdHolder> = match serde_xml_rs::from_str(&wrapper_str) {
		Ok(d) => d,
		Err(err) => return Err(Error::new(ErrorKind::Other, format!("Cannot deserialize Wrapper struct for ItemView data. {err}"))),
	};
	Ok(wrapper.return_data())
}

pub fn load_alchemy() -> io::Result<Vec<osrs::DataHolder>> {
	let mut path = get_local_data_dir()?;
	path.push(FAVDATA_DIR);
	if !check_dir(&path) {
		return Err(Error::new(ErrorKind::Other, "Alchemy saved items path doesn't exist"));
	}
	path.push(ALCH_FILE);
	let mut file = fs::OpenOptions::new().read(true).open(&path)?;
	let mut wrapper_str = String::new();
	let _ = file.read_to_string(&mut wrapper_str)?;
	let wrapper: Wrapper<osrs::DataHolder> = match serde_xml_rs::from_str(&wrapper_str) {
		Ok(d) => d,
		Err(err) => return Err(Error::new(ErrorKind::Other, format!("Cannot deserialize Wrapper struct for Alchemy data. {err}"))),
	};
	Ok(wrapper.return_data())
}

pub fn load_recipe(id: usize) -> io::Result<RecipeHolder> {
	let vec = load_recipes_vec()?;
	let mut target: Option<String> = None;
	for files in vec.iter(){
		let id_offset = files.find(' ').unwrap_or(files.len());
		let file_id = files[..id_offset].to_string().parse::<usize>().unwrap_or(usize::MAX);
		if id == file_id {
			target = Some(files[id_offset + 1..].to_string());
			break;
		}
	}
	if let None = target { return Err(Error::new(ErrorKind::Other, format!("Cannot find file with id {id}"))); }
	let mut target = target.unwrap();
	let mut path = get_local_data_dir()?;
	path.push(RECIPES_DIR);
	path.push(format!("{}-{}.xml", id, target));
	let mut file = fs::OpenOptions::new().read(true).open(&path)?;
	target.clear();
	let _ = file.read_to_string(&mut target)?;
	let data: RecipeHolder = match serde_xml_rs::from_str(&target) {
		Ok(d) => d,
		Err(err) => return Err(Error::new(ErrorKind::Other, format!("Cannot deserialize RecipeHolder struct. {err}"))),
	};
	Ok(data)
}

pub fn load_config() -> io::Result<ConfigSettings>{
	let mut path = get_local_data_dir()?;
	path.push(CONFIG_DIR);
	if !check_dir(&path) {
		return Err(Error::new(ErrorKind::Other, "Config path doesn't exist"));
	}
	path.push(CONFIG_FILE);
	let mut file = fs::OpenOptions::new().read(true).open(&path)?;
	let mut wrapper_str = String::new();
	let _ = file.read_to_string(&mut wrapper_str)?;
	let wrapper: ConfigSettings = match serde_xml_rs::from_str(&wrapper_str) {
		Ok(d) => d,
		Err(err) => return Err(Error::new(ErrorKind::Other, format!("Cannot deserialize ConfigSettings struct. {err}"))),
	};
	Ok(wrapper)
}

pub fn delete_recipe(id: usize) -> io::Result<()> {
	let vec = load_recipes_vec()?;
	let mut target: Option<String> = None;
	for files in vec.iter(){
		let id_offset = files.find(' ').unwrap_or(files.len());
		let file_id = files[..id_offset].to_string().parse::<usize>().unwrap_or(usize::MAX);
		if id == file_id {
			target = Some(files[id_offset + 1..].to_string());
			break;
		}
	}
	if let None = target { return Err(Error::new(ErrorKind::Other, format!("Cannot find file with id {id}"))); }
	let target = target.unwrap();
	let mut path = get_local_data_dir()?;
	path.push(RECIPES_DIR);
	path.push(format!("{}-{}.xml", id, target));
	fs::remove_file(path)?;
	Ok(())
}

pub fn delete_all_recipes() -> io::Result<()> {
	let mut path = get_local_data_dir()?;
	path.push(RECIPES_DIR);
	for entry in fs::read_dir(&path)? {
		let entry = entry?;
		fs::remove_file(entry.path())?;
	}
	Ok(())
}

pub fn delete_item_view() -> io::Result<()> {
	let mut path = get_local_data_dir()?;
	path.push(FAVDATA_DIR);
	path.push(ITEM_FILE);
	fs::remove_file(path)?;
	Ok(())
}

pub fn delete_alchemy() -> io::Result<()> {
	let mut path = get_local_data_dir()?;
	path.push(FAVDATA_DIR);
	path.push(ALCH_FILE);
	fs::remove_file(path)?;
	Ok(())
}


pub fn get_local_data_dir() -> io::Result<path::PathBuf> {
	let path = dirs::data_dir().ok_or(Error::new(ErrorKind::Other, "Cannot get user data dir"))?;
	let mut path = path::PathBuf::from(path);
	path.push(DATA_DIR);
	return Ok(path);
}
