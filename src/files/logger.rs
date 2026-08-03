use std::io::{self, Error, ErrorKind, Write};
use std::{path, fs};

use chrono;

use super::{check_dir, create_dir};
use super::{DATA_DIR, LOGGER_DIR, LOG_FILE_NAME};

/// Custom macro used to generate Message log messages. This macro is exported to base crate module
/// so no additional imports are required.  
/// Basic messages and format messages are implemented
/// ```rust
/// log_mess!["This is a message"];
/// log_mess!["This is also a message with extra information: {}", 2137];
/// ```
#[macro_export]
macro_rules! log_mess {
	($mesg:expr) => {
		{
			use crate::files::log_message_str;
			let _ = log_message_str(format!("{} | {} {}:{} - {}",
				chrono::Local::now().format("%d.%m.%Y %H:%M:%S:%6f"),
				std::file![], 
				std::line![], 
				std::column![], 
				$mesg));
		}
	};
	
	($fmt:expr, $($mesg:expr),+) => {
		{
			use crate::files::log_message_str;
			let _ = log_message_str(format!("{} | {} {}:{} - {}",
				chrono::Local::now().format("%d.%m.%Y %H:%M:%S:%6f"),
				std::file![], 
				std::line![], 
				std::column![], 
				format!($fmt, $($mesg),+)));
		}
	};
}

/// Custom macro used to generate Error log messages. This macro is exported to base crate module
/// so no additional imports are required. For now the only difference is big ERROR text before the
/// log message.  
/// Basic messages and format messages are implemented
/// ```rust
/// log_err!["This is a message"];
/// log_err!["This is also a message with extra information: {}", 0];
/// ```
#[macro_export]
macro_rules! log_err {
	($mesg:expr) => {
		{
			use crate::files::log_error_str;
			let _ = log_error_str(format!("{} | {} {}:{} - {}",
				chrono::Local::now().format("%d.%m.%Y %H:%M:%S:%6f"),
				std::file![], 
				std::line![], 
				std::column![], 
				$mesg));
		}
	};
	
	($fmt:expr, $($mesg:expr),+) => {
		{
			use crate::files::log_error_str;
			let _ = log_error_str(format!("{} | {} {}:{} - {}",
				chrono::Local::now().format("%d.%m.%Y %H:%M:%S:%6f"),
				std::file![], 
				std::line![], 
				std::column![], 
				format!($fmt, $($mesg),+)));
		}
	};
}

/// Function used to prepare log directory and file. Checks the existing files and deletes the
/// oldest one if the limit is reached.
pub fn setup() -> io::Result<()> {
	let path = dirs::data_dir().ok_or(Error::new(ErrorKind::Other, "Cannot get user data dir"))?;
	let mut path = path::PathBuf::from(path);
	path.push(DATA_DIR);
	path.push(LOGGER_DIR);
	if !check_dir(&path) {
		create_dir(&path)?;
	}
	let log_vec = get_logs_files_vec(&path)?;
  dbg!(&log_vec);
	if !log_vec.is_empty() && log_vec.len() >= 5 {
		let mut to_del = String::new();
		let mut date_to_del = chrono::NaiveDateTime::MAX;
		for string in log_vec.iter() {
      println!["{}", string];
			let dt = chrono::NaiveDateTime::parse_from_str(&string, "%d_%m_%Y_%H_%M");
			if dt.is_err() {
        println!("{}", dt.unwrap_err());
				to_del = string.clone();
				break;
			}
			let dt = dt.unwrap();
			if dt < date_to_del { date_to_del = dt.into(); }
		}
		if to_del.is_empty() {
			to_del = format!("{}", date_to_del.format("%d_%m_%Y_%H_%M"));
		}
		delete_log(to_del)?;
	}
	let file_name = chrono::Local::now().format("%d_%m_%Y_%H_%M").to_string();
	let _ = LOG_FILE_NAME.set(file_name.clone());
	path.push(format!("{}.logos", file_name));
	let _ = fs::OpenOptions::new().write(true).create(true).open(&path)?;
	Ok(())
}

/// Writes Message log message to log file. Returns standard std::io::Result struct.
pub fn log_message_str(mut string: String) -> io::Result<()> {
	string += "\n";
	print!("{}", string);
	let path = dirs::data_dir().ok_or(Error::new(ErrorKind::Other, "Cannot get user data dir"))?;
	let mut path = path::PathBuf::from(path);
	path.push(DATA_DIR);
	path.push(LOGGER_DIR);
	path.push(format!("{}.logos", LOG_FILE_NAME.get().unwrap()));
	let mut file = fs::OpenOptions::new().append(true).open(&path)?;
	file.write(&string.into_bytes())?;
	Ok(())
}

/// Writes Error log message to log file. Returns standard std::io::Result struct.
pub fn log_error_str(mut string: String) -> io::Result<()> {
	string += "\n";
	eprint!("{}", string);
	let path = dirs::data_dir().ok_or(Error::new(ErrorKind::Other, "Cannot get user data dir"))?;
	let mut path = path::PathBuf::from(path);
	path.push(DATA_DIR);
	path.push(LOGGER_DIR);
	path.push(format!("{}.logos", LOG_FILE_NAME.get().unwrap()));
	let mut file = fs::OpenOptions::new().append(true).open(&path)?;
	file.write(&string.into_bytes())?;
	Ok(())
}

/// Function used to get Vec<String> struct containing all files inside log directory. The output is
/// wrapped with standard std::io::Result<>.
fn get_logs_files_vec(path: &path::PathBuf) -> io::Result<Vec<String>> {
	let mut vec = vec![];
	for entry in fs::read_dir(&path)? {
		let entry = entry?;
		let val = match entry.file_name().into_string() {
			Ok(mut string) => {
				string = string.replace('-', " ");
				let str_offset = string.find(".logos").unwrap_or(string.len());
				string[..str_offset].to_string()
			}
			Err(_) => {
				eprintln!("Cannot parse OsStrint to String... stopping the loop");
				break;
			}
		};
		vec.push(val);
	}
	Ok(vec)
}

/// Deletes target file.
fn delete_log(file: String) -> io::Result<()> {
  println!["To del: {}", file];
	let path = dirs::data_dir().ok_or(Error::new(ErrorKind::Other, "Cannot get user data dir"))?;
	let mut path = path::PathBuf::from(path);
	path.push(DATA_DIR);
	path.push(LOGGER_DIR);
	path.push(format!("{}.logos", file));
	fs::remove_file(path)?;
	Ok(())
}
