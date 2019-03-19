use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;

const path_to_all_songs: &str = "static/songs/all_songs";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
	pub name: String,
	pub path: String,
	pub duration: u16,
}
// TODO: get rid of so many clones in my codebase,
// find a way to return random song straight from the json
// instead of serializing big vectors of songs
pub fn get_random_song() -> Song {
	let mut available_songs = read_available_songs();
	let mut rng = rand::thread_rng();
	if let Some(song) = available_songs.choose(&mut rng).cloned() {
		song
	} else {
		return Song {
			path: "xd".to_owned(),
			name: "xd".to_owned(),
			duration: 30,
		};
	}
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

fn read_available_songs() -> Vec<Song> {
	let json_path = get_json_path(path_to_all_songs);
	let file = fs::File::open(&json_path);
	match file {
		Ok(file) => {
			let reader = BufReader::new(file);
			let songs_available: Vec<Song> = serde_json::from_reader(reader).unwrap();
			songs_available
		}
		_ => Vec::<Song>::new(),
	}
}

/// add song to json containing all available songs
/// in case if user doesn't specify next song to be played
/// a next song will be read from this list
fn add_song_to_list(song: &Song) {
	let json_path = get_json_path(path_to_all_songs);
	let mut songs_available = read_available_songs();
	songs_available.push(song.clone());
	println!("{:#?}", songs_available);
	let songs_available_json = serde_json::to_vec(&songs_available).unwrap();
	fs::write(json_path, songs_available_json).unwrap();
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
			// add song to list of available songs on disc
			add_song_to_list(&song);
			Ok(song)
		}
		_ => Err(()),
	}
}
