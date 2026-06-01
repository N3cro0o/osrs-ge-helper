// TODO merge logger and save mod into file_handle mod

use std::io::{self, Error, ErrorKind, Write};
use std::{path, fs};

use chrono;

use super::{check_dir, create_dir};
use super::{DATA_DIR, LOGGER_DIR, LOG_FILE_NAME};

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
	
	($fmt:expr, $mesg:expr) => {
		{
			use crate::files::log_message_str;
			let _ = log_message_str(format!("{} | {} {}:{} - {}",
				chrono::Local::now().format("%d.%m.%Y %H:%M:%S:%6f"),
				std::file![], 
				std::line![], 
				std::column![], 
				format!($fmt, $mesg)));
		}
	};
}

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
	
	($fmt:expr, $mesg:expr) => {
		{
			use crate::files::log_error_str;
			let _ = log_error_str(format!("{} | {} {}:{} - {}",
				chrono::Local::now().format("%d.%m.%Y %H:%M:%S:%6f"),
				std::file![], 
				std::line![], 
				std::column![], 
				format!($fmt, $mesg)));
		}
	};
}

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
