use crate::song::Song;
use crate::song_queue::SongQueue;
use actix::*;
use std::path::Path;
use std::process::Command;
use std::{thread, time};

#[derive(Default)]
pub struct Radio {
    script_path: String,
}

impl Radio {
    pub fn new() -> Self {
        // panic if script isn't avialable
        let script_path = get_script_path().unwrap();
        Radio { script_path }
    }

    fn play_song(&self, song_path: &str, song_duration: i32) {
        println!("sleeping");
        let timeout = time::Duration::from_secs(5);
        thread::sleep(timeout);
        println!("done sleeping");
        println!("{:#?}", song_path);
        let handle = Command::new("timeout")
            .arg(&song_duration.to_string())
            .arg(self.script_path.clone())
            .arg("--freq")
            .arg("104.1")
            .arg("--audio")
            .arg(song_path)
            .output()
            .unwrap();
        //   println!("{:#?}", String::from_utf8(handle.stderr.clone()));
        //   println!("{:#?}", String::from_utf8(handle.stdout.clone()));
    }
}

pub struct PlaySong {
    pub song: Song,
    pub queue_addr: Addr<SongQueue>,
}

pub struct NextSong;
impl Message for NextSong {
    type Result = ();
}

impl Message for PlaySong {
    type Result = ();
}

impl Actor for Radio {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {}
}

impl Handler<PlaySong> for Radio {
    type Result = ();
    fn handle(&mut self, msg: PlaySong, ctx: &mut Self::Context) -> Self::Result {
        // let timeout = time::Duration::from_secs(msg.song.duration as u64);
        self.play_song(&msg.song.path, msg.song.duration);
        msg.queue_addr.do_send(NextSong {});
    }
}

pub fn get_script_path() -> Result<String, ()> {
    let path = Path::new("../fm_transmitter/fm_transmitter");
    let script_exists = path.exists();
    if script_exists {
        Ok(std::fs::canonicalize(&path)
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned())
    } else {
        Err(())
    }
}
