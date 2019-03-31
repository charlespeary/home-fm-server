use super::io::MyIO;
use super::radio::Radio;
use super::song::Song;
use crate::client_publisher::ClientPublisher;
use crate::db::{CheckSongExistence, DBExecutor, SaveSong};
use crate::io::IOJob::DownloadSong;
use crate::song::{GetRandomSong, NewSong, SongRequest};
use crate::web_socket::UserMessage;
use actix::fut::{wrap_future, Either};
use actix::*;
use futures::future::{join_all, ok as fut_ok, result, Future};
use std::{thread, time};

pub struct SongQueue {
    pub IO: Addr<MyIO>,
    pub db: Addr<DBExecutor>,
    pub songs_queue: Vec<Song>,
    pub radio: Addr<Radio>,
}

impl SongQueue {}

impl Actor for SongQueue {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.next_song(ctx);
    }
}

#[derive(Message, Debug)]
pub enum QueueJob {
    PlaySong { song: Song },
    NoSongAvailable,
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
        let five_secs = time::Duration::from_secs(5);
        thread::sleep(five_secs);
        self.next_song(ctx);
    }

    fn handle_activities(&mut self, ctx: &mut Context<SongQueue>, radio_job: QueueJob) {
        println!("Incoming activity - {:#?}", radio_job);
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
            QueueJob::NoSongAvailable => (),
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
            // println!("{:#?}", self.songs_queue);
            //  println!("there was a song in queue, let's play it now");
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

// TONS OF TRAIL AND ERROR CODE THAT MIGHT BE USEFUL IN NEXT ITERATIONS, SO I AM LEAVING IT AS IT IS FOR NOW

//ctx.spawn(
//future
//.map(|song, actor, ctx| {
//if let Ok(song) = song {
//self.handle_activities(ctx, QueueJob::ScheduleSong { song });
//()
//} else {
//let future = wrap_future(
//actor
//.IO
//.send(IOJob::DownloadSong {
//song_name: requested_song.get_name(),
//})
//.from_err()
//.and_then(|song| {
//let song = song.unwrap();
//actor.db.send(SaveSong { song })
//})
//.and_then(|song| {
//let song = song.unwrap();
//self.handle_activities(ctx, QueueJob::ScheduleSong { song });
//let response = UserMessage::<Song> {
//success: true,
//action: "start_song_download".to_owned(),
//value: song.clone(),
//};
//ClientPublisher::from_registry().send(response)
//}),
//);
//ctx.spawn(future.map_err(|a, c, x| {}));
//}
//})
//.map_err(|a, c, x| {}),
//);

//fn get_song(
//    requested_song: &SongRequest,
//    ctx: &mut Context<SongQueue>,
//    actor: &mut SongQueue,
//) -> impl Future<Item = Song, Error = MailboxError> {
//    actor
//        .IO
//        .send(IOJob::DownloadSong {
//            song_name: requested_song.get_name(),
//        })
//        .map_err(|e| eprintln!("xD"))
//        .flatten()
//        .and_then(|song| {
//            actor.db.send(SaveSong { song }).from_err().map(|song| {
//                if let Ok(song) = song {
//                    song
//                } else {
//                    Song {
//                        id: 1,
//                        path: "too".to_owned(),
//                        duration: 150,
//                        name: "xdd".to_owned(),
//                    }
//                }
//            })
//        })
//}

//                    match song {
//                        // song is already in the database, let's add it to the queue
//                        Ok(song) => actor.handle_activities(ctx, QueueJob::ScheduleSong { song }),
//                        // song doesn't exist, let's download it and save into the database
//                        Err(e) => self.IO.send(IOJob::DownloadSong {
//                            song_name: requested_song.get_name(),
//                        }),
//                    }

//        let future = wrap_future::<_, Self>(self.db.send(CheckSongExistence {
//            song_name: requested_song.get_name(),
//        }));
//        ctx.spawn(future.map(|song, actor, ctx| {
//            if let Ok(song) = song {
//                Either::A(fut_ok(song))
//            } else {
//                    self.IO
//                        .send(IOJob::DownloadSong {
//                            song_name: requested_song.get_name(),
//                        })
//                        .map(|new_song| {
//                            let song = new_song.unwrap();
//                            actor
//                                .db
//                                .send(SaveSong { song })
//                                .and_then(|song| Either::B(fut_ok(song.unwrap())))
//                        })
//                // should I spawn context?
//            }
//        }).and_then(|res| // do something with song));
//    }

//        let download_song = wrap_future::<_, Self>(self.IO.send(DownloadSong {
//            song_name: requested_song.get_name(),
//        }));
//
//        let song = download_song.map(|song, actor, ctx| song.unwrap());
//        let save_song =
//            song.map(|song, actor, ctx| wrap_future::<_, Self>(actor.db.send(SaveSong { song })));
//        let saved_song = save_song.map(|res, actor, ctx| {
//            res.map(|song, actor, ctx| {
//                let song = song.unwrap();
//                actor.handle_activities(ctx, QueueJob::ScheduleSong { song: song.clone() });
//                println!("???");
//                let response = UserMessage::<Song> {
//                    success: true,
//                    action: "song_download_finished".to_owned(),
//                    value: song,
//                };
//                ClientPublisher::from_registry().send(response)
//            })
//        });
//        ctx.spawn(
//            saved_song
//                .map(|a, c, x| {})
//                .map_err(|e, a, c| println!("something went wrong")),
//        );
