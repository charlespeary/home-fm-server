use super::io::MyIO;
use super::song::Song;
use super::web_socket::MyWebSocket;
use actix::*;
use std::{thread, time};

#[derive(Message)]
pub enum RadioResponse {
    NextSong,
}

pub struct Radio {
    pub IO: Addr<MyIO>,
}

impl Radio {}

impl Actor for Radio {
    type Context = Context<Self>;
}

#[derive(Message)]
pub enum RadioJob {
    PlaySong {
        song: Song,
        ws_addr: Addr<MyWebSocket>,
    },
    NoSongAvailable,
}

impl Handler<RadioJob> for Radio {
    type Result = ();
    fn handle(&mut self, msg: RadioJob, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            RadioJob::PlaySong { song, ws_addr } => {
                self.play_song(&song);
                ws_addr.do_send(RadioResponse::NextSong)
            }
            RadioJob::NoSongAvailable => (),
        }
    }
}

// fn get_script_path() -> String {
// 	let static_path = PathBuf::from("fm_transmitter-master");
// 	format!("{}/PiStation.py", static_path.display())
// }

impl Radio {
    pub fn play_song(&self, song: &Song) {
        let five_secs = time::Duration::from_secs(5);
        thread::sleep(five_secs);
    }
}
