#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[macro_use]
mod song;
mod io;
mod radio;
mod system;
mod web_socket;
use song::play_song;
use system::System;
extern crate num_cpus;

fn main() {
    play_song("/home/sniadek/Projects/home-fm-server/static/songs/BIAŁAS & LANEK - Blizny na rękach [official audio].mp3");
    let system = System::new();
}
