mod save;
#[macro_use]
mod logger;

use std::io;
use std::{path, fs};
use std::sync::OnceLock;

use crate::structs::RecipeHolder;
use crate::osrs;

pub const DATA_DIR: &str = "OSRE-calculator";
pub const RECIPES_DIR: &str = "recipes";
pub const FAVDATA_DIR: &str = "favdata";
pub const LOGGER_DIR: &str = "logs";

pub const ITEM_FILE: &str = "view_data.xml";
pub const ALCH_FILE: &str = "alch_data.xml";

static LOG_FILE_NAME: OnceLock<String> = OnceLock::new();

pub fn save_recipe(data: &RecipeHolder) -> io::Result<()> {
	save::save_recipe(data)
}

pub fn load_recipes_vec() -> io::Result<Vec<String>> {
	save::load_recipes_vec()
}

pub fn load_recipe(id: usize) -> io::Result<RecipeHolder> {
	save::load_recipe(id)
}

pub fn check_dir(path: &path::PathBuf) -> bool {
	use path::Path;
	Path::new(path).exists()
}

pub fn create_dir(path: &path::PathBuf) -> io::Result<()> {
	fs::create_dir_all(path)
}

pub fn delete_recipe(id: usize) -> io::Result<()> {
	save::delete_recipe(id)
}

pub fn setup_logger() -> io::Result<()> {
	logger::setup()
}

pub fn log_message_str(string: String) -> io::Result<()> {
	logger::log_message_str(string)
}

pub fn log_error_str(string: String) -> io::Result<()> {
	logger::log_error_str(string)
}

pub fn save_view_items(data: &Vec<osrs::DataHolder>) -> io::Result<()> {
	save::save_view_items(data)
}

pub fn load_view_items() -> io::Result<Vec<osrs::DataHolder>> {
	save::load_view_items()
}

pub fn save_alchemy(data: &Vec<osrs::DataHolder>) -> io::Result<()> {
	save::save_alchemy(data)
}

pub fn load_alchemy() -> io::Result<Vec<osrs::DataHolder>> {
	save::load_alchemy()
}

pub fn get_local_data_dir() -> io::Result<path::PathBuf> {
	save::get_local_data_dir()
}

pub fn delete_all_data() -> io::Result<()> {
	save::delete_all_recipes()?;
	save::delete_item_view()?;
	save::delete_alchemy()?;
	Ok(())
}