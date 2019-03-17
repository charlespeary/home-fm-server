use super::io::MyIO;
use super::song::{play_song, Song};
use actix::*;
use std::time::Duration;
pub struct Radio {
    pub song_queue: Vec<Song>,
    pub IO: Addr<MyIO>,
}

impl Radio {}

impl Actor for Radio {
    type Context = SyncContext<Self>;
}

#[derive(Message)]
pub enum RadioJob {
    PlaySong { song: Song },
}

impl Handler<RadioJob> for Radio {
    type Result = ();
    fn handle(&mut self, msg: RadioJob, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            RadioJob::PlaySong { song } => {}
        }
    }
}
