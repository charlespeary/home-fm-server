use super::io::MyIO;
use super::radio::Radio;
use super::song::Song;
use crate::client_publisher::ClientPublisher;
use crate::db::{DBExecutor, SaveSong};
use crate::io::IOJob::DownloadSong;
use crate::radio;
use crate::song::{GetRandomSong, SongRequest};
use crate::web_socket::UserMessage;
use actix::fut::wrap_future;
use actix::*;
use futures::future::Future;

pub struct SongQueue {
    pub IO: Addr<MyIO>,
    pub db: Addr<DBExecutor>,
    pub songs_queue: Vec<Song>,
    pub radio: Addr<Radio>,
}

impl Actor for SongQueue {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.next_song(ctx);
    }
}

#[derive(Message, Debug)]
pub enum QueueJob {
    PlaySong { song: Song },
    ScheduleSong { song: Song },
    DownloadSong { requested_song: SongRequest },
}

impl Handler<QueueJob> for SongQueue {
    type Result = ();
    fn handle(&mut self, msg: QueueJob, ctx: &mut Self::Context) -> Self::Result {
        self.handle_activities(ctx, msg);
    }
}

impl SongQueue {
    pub fn play_song(&mut self, ctx: &mut Context<SongQueue>, song: &Song) {
        self.radio.do_send(radio::PlaySong {
            song: song.clone(),
            queue_addr: ctx.address(),
        });
    }

    fn handle_activities(&mut self, ctx: &mut Context<SongQueue>, radio_job: QueueJob) {
        //    println!("Incoming activity - {:#?}", radio_job);
        match radio_job {
            QueueJob::PlaySong { song } => {
                self.play_song(ctx, &song);
                let response = UserMessage::<Song> {
                    success: true,
                    action: "play_song".to_owned(),
                    value: song.clone(),
                };
                ClientPublisher::from_registry().do_send(response);
            }
            QueueJob::DownloadSong { requested_song } => {
                self.download_song(ctx, requested_song);
            }
            QueueJob::ScheduleSong { song } => {
                self.songs_queue.push(song);
            }
        }
    }

    //TODO : Something is blocking db, probably function that download song
    fn next_song(&mut self, ctx: &mut Context<SongQueue>) {
        if let Some(song) = self.songs_queue.first() {
            self.handle_activities(ctx, QueueJob::PlaySong { song: song.clone() });
            self.songs_queue.remove(0);
        } else {
            //  println!("just playing some random stuff");
            let future = actix::fut::wrap_future::<_, Self>(self.db.send(GetRandomSong {}));
            ctx.spawn(
                future //
                    .map(move |res, actor, ctx| {
                        let song = res.unwrap();
                        actor.handle_activities(ctx, QueueJob::PlaySong { song });
                    })
                    .map_err(|e, a, c| println!("something went wrong")),
            );
        }
    }

    fn download_song(&mut self, ctx: &mut Context<SongQueue>, requested_song: SongRequest) {
        ctx.spawn(
            wrap_future::<_, Self>(self.IO.send(DownloadSong {
                song_name: requested_song.get_name(),
            }))
            .map(|song, actor, ctx| {
                let song = song.unwrap();
                actor.db.send(SaveSong { song }).map(|song| song.unwrap())
            })
            .and_then(|res, actor, ctx| wrap_future(res))
            .and_then(|song, actor, ctx| {
                actor.handle_activities(ctx, QueueJob::ScheduleSong { song: song.clone() });
                let response = UserMessage::<Song> {
                    success: true,
                    action: "song_download_finished".to_owned(),
                    value: song,
                };
                wrap_future(ClientPublisher::from_registry().send(response))
            })
            .map_err(|e, a, c| println!("db crashed - {:#?}", e)),
        );
    }
}

impl Handler<radio::NextSong> for SongQueue {
    type Result = ();
    fn handle(&mut self, msg: radio::NextSong, ctx: &mut Self::Context) -> Self::Result {
        self.next_song(ctx);
    }
}
