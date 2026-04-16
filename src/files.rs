mod save;
#[macro_use]
mod logger;

use std::io;
use std::{path, fs};
use std::sync::OnceLock;

use crate::structs::RecipeHolder;

pub const DATA_DIR: &str = "OSRE-calculator";
pub const RECIPES_DIR: &str = "recipes";
pub const LOGGER_DIR: &str = "logs";

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
	use std::path::Path;
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