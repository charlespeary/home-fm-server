use super::schema::songs;
use chrono::prelude::*;
use diesel::{Insertable, Queryable};
use serde::{self, Deserialize, Serialize, Serializer};
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SongRequest {
    pub artists: String,
    pub name: String,
    #[serde(skip_deserializing, default = "now")]
    pub requested_at: DateTime<Utc>,
    thumbnail_url: String,
}

fn now() -> DateTime<Utc> {
    Utc::now()
}

impl SongRequest {
    pub fn get_formatted_name(&self) -> String {
        format!("{} - {}", self.name, self.artists)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
pub struct Song {
    id: i32,
    pub name: String,
    #[serde(skip_serializing)]
    pub path: String,
    pub duration: i32,
    thumbnail_url: String,
    artists: String,
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "songs"]
pub struct NewSong {
    pub name: String,
    path: String,
    duration: i32,
    thumbnail_url: String,
    // , separated array
    pub artists: String,
}

pub struct GetRandomSong;

pub fn get_song_path(song_name: &str) -> String {
    let canonicalized_path = std::fs::canonicalize(PathBuf::from("static/songs")).unwrap();
    format!("{}/{}", canonicalized_path.display(), song_name)
}

fn get_json_path(song_path: &str) -> String {
    format!("{}.info.json", song_path)
}

// returns boolean - whether song was downloaded or not
pub fn download_song(requested_song: &SongRequest) -> Result<NewSong, ()> {
    let song_path = get_song_path(&requested_song.get_formatted_name());
    let search_query: &str = &format!("ytsearch1:{}", &requested_song.get_formatted_name());
    println!("{}, {}", song_path, &requested_song.get_formatted_name());
    let output = Command::new("youtube-dl")
        // download one song from youtube
        .current_dir("./static/songs")
        .arg(search_query)
        // extract audio from the video and format it to mp3
        .arg("-x")
        .arg("--audio-format")
        .arg("wav")
        .arg("--output")
        // why not just use song_path? without %(ext)s weird things happen inside youtube-dl and it outputs not working on rpi working file
        .arg(format!(
            "{}.%(ext)s",
            &requested_song.get_formatted_name().clone()
        ))
        .arg("--write-info-json")
        .output();
    if output.is_ok() {
        let info = get_song_info(&song_path, &requested_song.name).unwrap();
        Ok(NewSong {
            duration: info.duration,
            name: requested_song.name.clone(),
            artists: requested_song.artists.clone(),
            thumbnail_url: requested_song.thumbnail_url.clone(),
            path: format!("{}.wav", song_path),
        })
    } else {
        println!(
            "Error during downloading a song - {:#?}",
            String::from_utf8(output.unwrap().stderr)
        );
        Err(())
    }
    // decode duration from .info.json that youtube-dl downloads
}

#[derive(Serialize, Deserialize)]
struct Info {
    duration: i32,
}

fn get_song_info(song_path: &str, song_name: &str) -> Result<Info, ()> {
    let json_path = get_json_path(song_path);
    let file = fs::File::open(&json_path);
    match file {
        Ok(file) => {
            let reader = BufReader::new(file);
            let json_content: Info = serde_json::from_reader(reader).unwrap();
            fs::remove_file(json_path);
            Ok(json_content)
        }
        _ => {
            println!("error during opening a file");
            Err(())
        }
    }
}
