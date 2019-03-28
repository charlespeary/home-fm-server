use super::schema::songs;
use diesel::{Insertable, Queryable};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;
const PATH_TO_SONGS_LIST: &str = "static/songs/all_songs";

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
pub struct Song {
    id: i32,
    name: String,
    path: String,
    duration: i32,
}

#[derive(Insertable)]
#[table_name = "songs"]
pub struct NewSong {
    name: String,
    path: String,
    duration: i32,
}
pub struct GetRandomSong;

// TODO: get rid of so many clones in my codebase,
// find a way to return random song straight from the json
// instead of serializing big vectors of songs
pub fn get_random_song() -> Song {
    // number of available songs in json list
    let available_songs_len = count_songs_in_list();
    if available_songs_len == 0 {
        Song {
            path: "xd".to_owned(),
            name: "xd".to_owned(),
            duration: 30,
            id: 1,
        }
    } else {
        let random_index = rand::thread_rng().gen_range(0, available_songs_len);
        get_song_from_file(random_index)
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
            println!("Song was already downloaded");
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
        .output();
    if output.is_ok() {
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

fn create_lightweight_json(json_path: &str, song: &Song) {
    // remove old json from youtube-dl which is pretty big
    fs::remove_file(json_path).unwrap();
    // serialize song into vector of bytes
    let song = serde_json::to_vec(song).unwrap();
    // save new lightweight json
    fs::write(json_path, song).unwrap();
    //File::create(json_path).unwrap().write(&song);
}

fn count_songs_in_list() -> usize {
    let json_path = get_json_path(PATH_TO_SONGS_LIST);
    let file = fs::File::open(&json_path);
    match file {
        Ok(file) => BufReader::new(file).lines().count(),
        _ => 0,
    }
}

fn get_song_from_file(index: usize) -> Song {
    let json_path = get_json_path(PATH_TO_SONGS_LIST);
    let f = fs::File::open(json_path).unwrap();
    let f = BufReader::new(f);
    let raw_song = f.lines().nth(index).unwrap().unwrap();
    serde_json::from_str(&raw_song).unwrap()
}

/// add song to json containing all available songs
/// in case if user doesn't specify next song to be played
/// a next song will be read from this list
fn add_song_to_list(song: &Song) {
    let json_path = get_json_path(PATH_TO_SONGS_LIST);
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(json_path)
        .unwrap();
    let deserialized_song = serde_json::to_string(song).unwrap();
    if let Err(e) = writeln!(file, "{}", deserialized_song) {
        eprintln!("Couldn't write to file: {}", e);
    }
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
                id: 1,
            };
            create_lightweight_json(&json_path, &song);
            // add song to list of available songs on disc
            add_song_to_list(&song);
            Ok(song)
        }
        _ => Err(()),
    }
}
