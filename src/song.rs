use serde::{Deserialize, Serialize};
use std::fs::{remove_file, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;
#[derive(Serialize, Deserialize, Debug)]
pub struct Song {
	pub name: String,
	path: String,
	duration: u16,
}

/// returns boolean - whether song was downloaded or not
pub fn download_song(song_name: &str) -> Result<Song, ()> {
	println!("Downloading song");
	let static_path = PathBuf::from("static/songs");
	let song_path = format!("{}/{}.mp3", static_path.display(), song_name);
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

fn get_song_info(song_path: &str, song_name: &str) -> Result<Song, ()> {
	let json_path = format!("{}.info.json", song_path);
	let file = File::open(&json_path);
	match file {
		Ok(file) => {
			let reader = BufReader::new(file);
			let json_content: Info = serde_json::from_reader(reader).unwrap();
			// delete file, we don't need it anymore
			remove_file(&json_path).unwrap();
			Ok(Song {
				path: song_path.to_owned(),
				duration: json_content.duration,
				name: song_name.to_owned(),
			})
		}
		_ => Err(()),
	}
}
