use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{thread, time};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
	pub name: String,
	path: String,
	duration: u16,
}

pub fn get_song_path(song_name: &str) -> String {
	let static_path = PathBuf::from("static/songs");
	let unformated_path = format!("{}/{}.mp3", static_path.display(), song_name);
	unformated_path.replace(" ", "_")
}

fn get_json_path(song_path: &str) -> String {
	format!("{}.info.json", song_path)
}

fn get_script_path() -> String {
	let static_path = PathBuf::from("fm_transmitter-master");
	format!("{}/PiStation.py", static_path.display())
}

/// check if song is already downloaded
fn check_downloaded(song_path: &str) -> bool {
	// check if .mp3 file and .info.json of song exists
	Path::new(song_path).exists() && Path::new(&get_json_path(song_path)).exists()
}

fn read_song_from_json(song_path: &str) -> Result<Song, ()> {
	let file = fs::File::open(&get_json_path(&song_path));
	match file {
		Ok(file) => {
			let reader = BufReader::new(file);
			let song = serde_json::from_reader(reader).unwrap();
			Ok(song)
		}
		_ => Err(()),
	}
}

/// returns boolean - whether song was downloaded or not
pub fn download_song(song_name: &str) -> Result<Song, ()> {
	println!("Downloading song");
	let song_path = get_song_path(song_name);
	// if we've got song on the disc already then read it from json, serialize and return
	if check_downloaded(&song_path) {
		let song = read_song_from_json(&song_path);
		// return song if it was sucessfully read from json file otherwise continue
		if song.is_ok() {
			return Ok(song.unwrap());
		}
	}
	let search_query: &str = &format!("ytsearch1:{}", song_name);
	let output = Command::new("youtube-dl")
		// download one song from youtube
		.arg(search_query)
		// extract audio from the video and format it to mp3
		.arg("-x")
		.arg("--audio-format")
		.arg("mp3")
		// save file in /static/songs directory
		.arg(format!("-o{}", song_path))
		.arg("--write-info-json")
		.output()
		.unwrap();
	// decode duration from .info.json that youtube-dl downloads
	get_song_info(&song_path, song_name)
}

#[derive(Serialize, Deserialize)]
struct Info {
	duration: u16,
}

fn create_lightweight_json(json_path: &str, song: &Song) {
	// remove old json from youtube-dl which is pretty big
	fs::remove_file(json_path).unwrap();
	// serialize song into vector of bytes
	let song = serde_json::to_vec(song).unwrap();
	// save new lightweight json
	fs::write(json_path, song).unwrap();
	//File::create(json_path).unwrap().write(&song);
}

fn get_song_info(song_path: &str, song_name: &str) -> Result<Song, ()> {
	let json_path = get_json_path(song_path);
	let file = fs::File::open(&json_path);
	match file {
		Ok(file) => {
			let reader = BufReader::new(file);
			let json_content: Info = serde_json::from_reader(reader).unwrap();
			// create lightweight json
			let song = Song {
				path: song_path.to_owned(),
				duration: json_content.duration,
				name: song_name.to_owned(),
			};
			create_lightweight_json(&json_path, &song);
			Ok(song)
		}
		_ => Err(()),
	}
}

pub fn play_song(song_path: &str) {
	// let script_path = get_script_path();
	// let x = Command::new("python")
	// 	.arg(script_path)
	// 	.arg("-f")
	// 	.arg("102.0")
	// 	.arg(song_path)
	// 	.output()
	// 	.unwrap();
	// println!("{:#?}", String::from_utf8(x.stdout));
	// println!("{:#?}", String::from_utf8(x.stderr));

	let five_secs = time::Duration::from_secs(5);
	thread::sleep(five_secs);
}
