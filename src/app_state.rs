use actix_web::{HttpRequest, Json, Result, State};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
	current_song: Arc<Mutex<String>>,
}

impl AppState {
	pub fn new() -> Self {
		AppState {
			current_song: Arc::new(Mutex::new(String::new())),
		}
	}
}

impl Serialize for AppState {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut state = serializer.serialize_struct("AppState", 2)?;
		let current_song = self.current_song.as_ref().lock().unwrap().clone();
		state.serialize_field("current_song", &current_song)?;
		state.end()
	}
}

pub fn get_app_state(req: &HttpRequest<AppState>) -> Result<Json<AppState>> {
	Ok(Json(req.state().clone()))
}

#[derive(Serialize)]
pub struct MyResponse {
	success: bool,
	message: &'static str,
}

#[derive(Serialize, Deserialize)]
pub struct Song {
	name: String,
}

// TODO: Make state shared across all threads
pub fn set_current_song(song: Json<Song>, state: State<AppState>) -> Result<Json<MyResponse>> {
	*(state.current_song.lock().unwrap()) = song.name.to_string();
	println!("current song: {}", state.current_song.lock().unwrap());
	let download_result = download_song(&song.name);
	return match download_result {
		Ok(message) => Ok(Json(MyResponse {
			success: true,
			message,
		})),
		Err(message) => Ok(Json(MyResponse {
			success: false,
			message,
		})),
	};
}

/// Formats the sum of two numbers as string
fn download_song(song_name: &str) -> Result<&'static str, &'static str> {
	println!("Downloading song");
	let static_path = PathBuf::from("static/songs");
	let output = Command::new("youtube-dl")
		.arg("-f bestaudio")
		.arg(format!("-o {path}/%(title)s", path = static_path.display()))
		.arg("https://www.youtube.com/watch?v=ZadwPbKA9Mg")
		.output()
		.expect("Couldn't download the song.");
	Ok("Successfully downloaded song")
}
