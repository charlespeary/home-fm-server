use super::schema::songs;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;
const PATH_TO_SONGS_LIST: &str = "static/songs/all_songs";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SongRequest {
    artists: Vec<String>,
    name: String,
}

impl SongRequest {
    pub fn get_name(&self) -> String {
        let formatted_artists = self.artists.join(", ");
        format!("{} - {}", self.name, formatted_artists)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
pub struct Song {
    id: i32,
    pub name: String,
    path: String,
    pub duration: i32,
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "songs"]
pub struct NewSong {
    pub name: String,
    path: String,
    duration: i32,
}
pub struct GetRandomSong;

pub fn get_song_path(song_name: &str) -> String {
    let static_path = PathBuf::from("static/songs");
    let unformated_path = format!("{}/{}.mp3", static_path.display(), song_name);
    unformated_path.replace(" ", "_")
}

fn get_json_path(song_path: &str) -> String {
    format!("{}.info.json", song_path)
}

/// check if song is already downloaded

/// returns boolean - whether song was downloaded or not
pub fn download_song(song_name: &str) -> Result<NewSong, ()> {
    let song_path = get_song_path(song_name);
    let search_query: &str = &format!("ytsearch1:{}", song_name);
    println!("I just started downloading a song - {}", song_name);
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
        .output();
    if output.is_ok() {
        println!("Successfully downloaded a song - {}", song_name);
        get_song_info(&song_path, song_name)
    } else {
        Err(())
    }
    // decode duration from .info.json that youtube-dl downloads
}

#[derive(Serialize, Deserialize)]
struct Info {
    duration: i32,
}

fn get_song_info(song_path: &str, song_name: &str) -> Result<NewSong, ()> {
    let json_path = get_json_path(song_path);
    let file = fs::File::open(&json_path);
    match file {
        Ok(file) => {
            let reader = BufReader::new(file);
            let json_content: Info = serde_json::from_reader(reader).unwrap();
            fs::remove_file(json_path);
            // create lightweight json
            let song = NewSong {
                path: song_path.to_owned(),
                duration: json_content.duration,
                name: song_name.to_owned(),
            };

            Ok(song)
        }
        _ => Err(()),
    }
}
