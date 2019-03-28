use super::io::MyIO;
use super::song::Song;
use super::web_socket::MyWebSocket;
use crate::db::DBExecutor;
use crate::song::GetRandomSong;
use actix::*;
use futures::future::Future;
use std::sync::Arc;
use std::{thread, time};

#[derive(Message)]
pub enum RadioResponse {
    NextSong,
}

pub struct Radio {
    pub IO: Addr<MyIO>,
    pub db: Addr<DBExecutor>,
    pub songs_queue: Vec<Song>,
}

impl Radio {}

impl Actor for Radio {
    type Context = SyncContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.next_song(ctx);
    }
}

#[derive(Message)]
pub enum RadioJob {
    PlaySong {
        song: Song,
        ws_addr: Addr<MyWebSocket>,
    },
    NoSongAvailable,
    //    ScheduleSong {
    //        song_name: String,
    //    },
}

impl Handler<RadioJob> for Radio {
    type Result = ();
    fn handle(&mut self, msg: RadioJob, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            RadioJob::PlaySong { song, ws_addr } => {
                self.play_song(&song);
                //   ws_addr.do_send(RadioResponse::NextSong)
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

    fn next_song(&self, ctx: &mut SyncContext<Radio>) {
        if let Some(song) = self.songs_queue.first() {
        } else {
            let future = self.db.send(GetRandomSong {});
        }
    }
}

//fn next_song(&self, ctx: &mut <Self as Actor>::Context) {
//    let mut songs_queue = ctx.state().songs_queue.lock().unwrap();
//    if let Some(song) = songs_queue.first() {
//        let song = song.clone();
//        songs_queue.remove(0);
//        ctx.state().radio.do_send(RadioJob::PlaySong {
//            song: song.clone(),
//            ws_addr: ctx.address(),
//        });
//        drop(songs_queue);
//        //   send_next_song(ctx, &song);
//    } else {
//        drop(songs_queue);
//        println!("Future is coming!");
//        let future = ctx.state().db.send(GetRandomSong {});
//        // clone radio and websocket addresses in order to move them inside future closure
//        let radio_addr = ctx.state().radio.clone();
//        let ws_addr = ctx.address();
//        Arbiter::spawn(
//            future
//                .map(move |res| {
//                    let radio = radio_addr.clone();
//                    let mut ws_addr = ws_addr.clone();
//                    let song = res.unwrap();
//                    send_next_song(&ws_addr, &song);
//                    radio.do_send(RadioJob::PlaySong {
//                        song: song.clone(),
//                        ws_addr,
//                    });
//                })
//                .map_err(|e| println!("something went wrong")),
//        );
//    };
//}
