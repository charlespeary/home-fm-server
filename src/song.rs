use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize, Deserialize)]
pub struct Song {
	name: String,
}

/// returns boolean - whether song was downloaded or not
pub fn download_song(song_name: &str) -> bool {
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
		.output()
		.unwrap();

	output.status.success()
}
