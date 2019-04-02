use super::io::MyIO;
use super::radio::Radio;
use super::song::Song;
use crate::client_publisher::ClientPublisher;
use crate::db::{CheckSongExistence, DBExecutor, SaveSong};
use crate::io::IOJob::DownloadSong;
use crate::radio;
use crate::song::{GetRandomSong, SongRequest};
use crate::web_socket::UserMessage;
use actix::dev::Request;
use actix::fut::{wrap_future, FutureWrap, IntoActorFuture};
use actix::*;
use futures::future::{ok as fut_ok, Future};
use serde::Serialize;

// TODO: I might need SyncContext for scenarios in which many songs come at once
// Basically it's working fine, but it takes some time just for one actor
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

#[derive(Serialize, Clone)]
pub struct NextSongInfo {
    pub next_song: Song,
    pub songs_queue: Vec<Song>,
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
                let response = UserMessage::<NextSongInfo> {
                    success: true,
                    action: "play_song".to_owned(),
                    value: NextSongInfo {
                        next_song: song,
                        songs_queue: self.songs_queue.clone(),
                    },
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

    fn next_song(&mut self, ctx: &mut Context<SongQueue>) {
        if let Some(song) = self.songs_queue.first() {
            self.handle_activities(ctx, QueueJob::PlaySong { song: song.clone() });
            self.songs_queue.remove(0);
        } else {
            //  println!("just playing some random stuff");
            let future = wrap_future::<_, Self>(self.db.send(GetRandomSong {}));
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

    fn schedule_song(
        &mut self,
        ctx: &mut Context<SongQueue>,
        song: &Song,
    ) -> impl ActorFuture<Item = (), Error = MailboxError, Actor = SongQueue> {
        self.handle_activities(ctx, QueueJob::ScheduleSong { song: song.clone() });
        let response = UserMessage::<Song> {
            success: true,
            action: "song_download_finished".to_owned(),
            value: song.clone(),
        };
        println!("Sending response");
        wrap_future(ClientPublisher::from_registry().send(response))
    }

    fn get_song(
        &mut self,
        ctx: &mut Context<SongQueue>,
        requested_song: SongRequest,
    ) -> impl ActorFuture<Item = Song, Error = MailboxError, Actor = SongQueue> {
        wrap_future::<_, Self>(self.IO.send(DownloadSong {
            song_name: requested_song.get_name(),
        }))
        .and_then(|song, actor, ctx| {
            wrap_future(actor.db.send(SaveSong {
                song: song.unwrap(),
            }))
        })
        .and_then(|song, actor, ctx| (fut_ok(song.unwrap()).into_actor(actor)))
    }

    // TODO: Check if song is already downloaded
    fn download_song(&mut self, ctx: &mut Context<SongQueue>, requested_song: SongRequest) {
        ctx.spawn(
            wrap_future::<_, Self>(self.db.send(CheckSongExistence {
                song_name: requested_song.get_name(),
            }))
            .map(|song, actor, ctx| {
                if let Ok(song) = song {
                    let future: Box<
                        dyn ActorFuture<Item = Song, Error = MailboxError, Actor = SongQueue>,
                    > = Box::new(fut_ok(song).into_actor(actor));
                    future
                } else {
                    Box::new(actor.get_song(ctx, requested_song))
                }
            })
            .and_then(|res, actor, ctx| res.map(|song, a, c| song))
            .and_then(|song, actor, ctx| {
                actor.handle_activities(ctx, QueueJob::ScheduleSong { song: song.clone() });
                actor.schedule_song(ctx, &song)
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
