use actix_web::{HttpRequest, Json, Result, State};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{ChildStderr, ChildStdout, Command, Stdio};
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

pub fn set_current_song(song: Json<Song>, state: State<AppState>) -> Result<Json<MyResponse>> {
	*(state.current_song.lock().unwrap()) = song.name.to_string();
	println!("current song: {}", state.current_song.lock().unwrap());
	let download_result = download_song(&song.name);
	return match download_result {
		Ok(message) => Ok(Json(MyResponse {
			success: true,
			message,
		})),
		Err(err) => Ok(Json(MyResponse {
			success: false,
			message: match err {
				DownloadErr::NoSongFound => "Youtube-dl couldn't find the given song.",
				DownloadErr::ConnectionLost => {
					"Youtube-dl lost the connection while downloading the song."
				}
			},
		})),
	};
}

#[derive(Fail, Debug)]
pub enum DownloadErr {
	#[fail(display = "Youtube-dl couldn't find given song.")]
	NoSongFound,
	#[fail(display = "Connection lost during download of the song.")]
	ConnectionLost,
}

/// Formats the sum of two numbers as string
fn download_song(song_name: &str) -> Result<&'static str, DownloadErr> {
	println!("Downloading song");
	let static_path = PathBuf::from("static/songs");
	let search_query: &str = &format!("ytsearch1:{}", song_name);
	let output = Command::new("youtube-dl")
		// download one song from youtube
		.arg(search_query)
		// extract audio from the video and format it to mp3
		.arg("-x")
		.arg("--audio-format")
		.arg("mp3")
		// save file in /static/songs directory
		.arg(format!(
			"-o{path}/%(title)s.%(ext)s",
			path = static_path.display()
		))
		// create stdout,stderr pipeline to listen for changes
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.unwrap();
	// result of the function is built by stuff that happens in std
	read_std(output.stdout.unwrap(), output.stderr.unwrap())
}

// TODO: parse errors from stderr and match them with DownloadErr enum
fn read_std(stdout: ChildStdout, stderr: ChildStderr) -> Result<&'static str, DownloadErr> {
	// buffer to read from stdout
	let stdout_reader = BufReader::new(stdout);
	let stdout_lines = stdout_reader.lines();
	// buffer to read from stderr
	let stderr_reader = BufReader::new(stderr);
	let stderr_lines = stderr_reader.lines();

	let mut error_message = String::new();

	// simply print the std value, it's gonna be streamed through websockets to the client
	for line in stdout_lines {
		println!("Read stdout: {:?}", line);
	}

	for line in stderr_lines {
		// for now just build string containing error message
		match line {
			Ok(val) => error_message.push_str(&val),
			_ => (),
		}
	}

	if error_message.len() > 0 {
		Err(DownloadErr::NoSongFound)
	} else {
		Ok("Successfully downloaded the song!")
	}
}
