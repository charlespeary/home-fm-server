use crate::song::Song;
use crate::song_queue::SongQueue;
use actix::*;
use std::{thread, time};

#[derive(Default)]
pub struct Radio;

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
}

impl Handler<PlaySong> for Radio {
    type Result = ();
    fn handle(&mut self, msg: PlaySong, ctx: &mut Self::Context) -> Self::Result {
        // let timeout = time::Duration::from_secs(msg.song.duration as u64);
        let timeout = time::Duration::from_secs(10);
        thread::sleep(timeout);
        msg.queue_addr.do_send(NextSong {});
    }
}
