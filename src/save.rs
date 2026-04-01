const DATA_DIR: &str = "OSRE-calculator";
const RECIPES_DIR: &str = "recipes";

use std::io::{self, Error, ErrorKind, Write, Read};
use std::{path, fs};

use crate::structs::RecipeHolder;

pub fn save_recipe(data: &RecipeHolder) -> io::Result<()> {
	let path = dirs::data_dir().ok_or(Error::new(ErrorKind::Other, "Cannot get user data dir"))?;
	let mut path = path::PathBuf::from(path);
	path.push(DATA_DIR);
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

pub fn load_recipes_vec() -> io::Result<Vec<String>> {
	let mut vec = vec![];
	let path = dirs::data_dir().ok_or(Error::new(ErrorKind::Other, "Cannot get user data dir"))?;
	let mut path = path::PathBuf::from(path);
	path.push(DATA_DIR);
	path.push(RECIPES_DIR);
	if !check_dir(&path) {
		create_dir(&path)?;
	}
	for entry in fs::read_dir(&path)? {
		let entry = entry?;
		// let val = match entry.file_name().into_string() {
			// Ok(string) => {
				// let str_offset = string.find('-').unwrap_or(string.len());
				// let val = match string[..str_offset].parse::<usize>() {
					// Ok(i) => i,
					// Err(err) => {
						// eprintln!("Cannot parse String to usize {err}...continuing");
						// continue;
					// }
				// };
				// val
			// }
			// Err(_) => {
				// eprintln!("Cannot parse OsStrint to String... continuing");
				// continue;
			// }
		// };
		let val = match entry.file_name().into_string() {
			Ok(mut string) => {
				string = string.replace('-', " ");
				let str_offset = string.find(".xml").unwrap_or(string.len());
				string[..str_offset].to_string()
			}
			Err(_) => {
				eprintln!("Cannot parse OsStrint to String... continuing");
				continue;
			}
		};
		vec.push(val);
	}
	Ok(vec)
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
	let path = dirs::data_dir().ok_or(Error::new(ErrorKind::Other, "Cannot get user data dir"))?;
	let mut path = path::PathBuf::from(path);
	path.push(DATA_DIR);
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
	let path = dirs::data_dir().ok_or(Error::new(ErrorKind::Other, "Cannot get user data dir"))?;
	let mut path = path::PathBuf::from(path);
	path.push(DATA_DIR);
	path.push(RECIPES_DIR);
	path.push(format!("{}-{}.xml", id, target));
	fs::remove_file(path)?;
	Ok(())
}

fn check_dir(path: &path::PathBuf) -> bool {
	use std::path::Path;
	Path::new(path).exists()
}

fn create_dir(path: &path::PathBuf) -> io::Result<()> {
	fs::create_dir_all(path)
}
