use super::io::MyIO;
use super::radio::Radio;
use super::song::Song;
use crate::client_publisher::ClientPublisher;
use crate::db::{CheckSongExistence, DBExecutor, SaveSong};
use crate::io::IOJob::DownloadSong;
use crate::radio;
use crate::song::{GetRandomSong, SongRequest};
use crate::web_socket::{EmptyValue, UserMessage};
use actix::fut::{wrap_future, FutureWrap, IntoActorFuture};
use actix::*;
use actix_web::{dev::Handler as RouteHandler, App, HttpRequest, HttpResponse};
use chrono::prelude::*;
use chrono::Utc;
use futures::future::{ok as fut_ok, Future};
use serde::Serialize;

type ActorContext = Context<SongQueue>;

#[derive(Serialize, Clone, Debug)]
pub struct ScheduledSong {
    song: Song,
    requested_at: DateTime<Utc>,
}

pub struct SongQueue {
    pub IO: Addr<MyIO>,
    pub db: Addr<DBExecutor>,
    pub songs_queue: Vec<ScheduledSong>,
    pub radio: Addr<Radio>,
    pub active_song: Option<Song>,
}

impl Actor for SongQueue {
    type Context = ActorContext;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.next_song(ctx);
    }
}

#[derive(Message, Debug)]
pub enum QueueJob {
    PlaySong {
        song: Song,
    },
    ScheduleSong {
        song: Song,
        // this field determines when websocket got request from client
        // because IO might download songs at different times, I want to keep track when it was requested
        // to sort songs in queue in order requested by user,
        requested_at: DateTime<Utc>,
    },
    DownloadSong {
        requested_song: SongRequest,
    },
}

impl Handler<QueueJob> for SongQueue {
    type Result = ();
    fn handle(&mut self, msg: QueueJob, ctx: &mut ActorContext) -> Self::Result {
        self.handle_activities(ctx, msg);
    }
}

#[derive(Serialize, Clone)]
pub struct NextSong {
    pub next_song: Song,
    pub songs_queue: Vec<ScheduledSong>,
}

impl SongQueue {
    pub fn play_song(&mut self, ctx: &mut ActorContext, song: &Song) {
        self.radio.do_send(radio::PlaySong {
            song: song.clone(),
            queue_addr: ctx.address(),
        });
    }

    fn handle_activities(&mut self, ctx: &mut ActorContext, radio_job: QueueJob) {
        match radio_job {
            QueueJob::PlaySong { song } => {
                self.active_song = Some(song.clone());
                self.play_song(ctx, &song);
                let response = UserMessage::<NextSong> {
                    success: true,
                    action: "next_song".to_owned(),
                    value: NextSong {
                        next_song: song,
                        songs_queue: self.songs_queue.clone(),
                    },
                };
                ClientPublisher::from_registry().do_send(response);
            }
            QueueJob::DownloadSong { requested_song } => {
                self.download_song(ctx, requested_song);
            }
            QueueJob::ScheduleSong { song, requested_at } => {
                self.songs_queue.push(ScheduledSong { song, requested_at });
                // sort
                self.sort_songs();
            }
        }
    }

    // sort songs by time they were requested at
    fn sort_songs(&mut self) {
        self.songs_queue
            .sort_by(|a, b| a.requested_at.time().cmp(&b.requested_at.time()));
    }

    fn next_song(&mut self, ctx: &mut ActorContext) {
        if let Some(scheduled_song) = self.songs_queue.first() {
            self.handle_activities(
                ctx,
                QueueJob::PlaySong {
                    song: scheduled_song.song.clone(),
                },
            );
            self.songs_queue.remove(0);
        } else {
            //  println!("just playing some random stuff");
            let future = wrap_future::<_, Self>(self.db.send(GetRandomSong {}));
            ctx.spawn(
                future //
                    .map(move |res, actor, ctx| {
                        if let Ok(song) = res {
                            println!("Song found");
                            actor.handle_activities(ctx, QueueJob::PlaySong { song });
                        } else {
                            println!("no songs available");
                            let response = UserMessage::<EmptyValue> {
                                success: false,
                                action: "no_songs_available".to_owned(),
                                value: EmptyValue {},
                            };
                            ClientPublisher::from_registry().do_send(response);
                        }
                    })
                    .map_err(|e, a, c| println!("something went wrong")),
            );
        }
    }

    fn schedule_song(
        &mut self,
        ctx: &mut ActorContext,
        song: &Song,
        requested_at: DateTime<Utc>,
    ) -> impl ActorFuture<Item = (), Error = MailboxError, Actor = SongQueue> {
        self.handle_activities(
            ctx,
            QueueJob::ScheduleSong {
                song: song.clone(),
                requested_at,
            },
        );
        let response = UserMessage::<Song> {
            success: true,
            action: "song_download_finished".to_owned(),
            value: song.clone(),
        };
        wrap_future(ClientPublisher::from_registry().send(response))
    }

    fn get_song(
        &mut self,
        ctx: &mut ActorContext,
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

    fn download_song(&mut self, ctx: &mut ActorContext, requested_song: SongRequest) {
        let requested_at = requested_song.requested_at.clone();
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
            .and_then(move |song, actor, ctx| actor.schedule_song(ctx, &song, requested_at))
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

#[derive(Message)]
pub struct BroadcastState;

#[derive(Serialize, Clone)]
pub struct QueueState {
    pub active_song: Option<Song>,
    pub songs_queue: Vec<ScheduledSong>,
}

// broadcast queue state after receiving message from websocket that there's new connection
impl Handler<BroadcastState> for SongQueue {
    type Result = ();
    fn handle(&mut self, msg: BroadcastState, ctx: &mut Self::Context) -> Self::Result {
        let response = UserMessage::<QueueState> {
            success: true,
            action: "queue_state".to_owned(),
            value: QueueState {
                active_song: self.active_song.clone(),
                songs_queue: self.songs_queue.clone(),
            },
        };
        ClientPublisher::from_registry().do_send(response);
    }
}
