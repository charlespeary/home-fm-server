#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[macro_use]
mod song;
mod client_publisher;
mod config;
mod db;
mod io;
mod radio;
mod responses;
mod schema;
mod song_queue;
mod system;
mod web_socket;
use system::System;
extern crate num_cpus;
#[macro_use]
extern crate diesel;

fn main() {
    //  play_song("/home/sniadek/Projects/home-fm-server/static/songs/BIAŁAS & LANEK - Blizny na rękach [official audio].mp3");
    let system = System::new();
}
