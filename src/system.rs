use super::config::{get_config, update_config};
use super::io::MyIO;
use super::radio::Radio;
use super::song::{delete_song, get_all_songs, toggle_song_nsfw};
use super::song_queue::SongQueue;
use super::web_socket::ws_index;
use crate::db::{new_pool, DBExecutor};
use actix::prelude::*;
use actix::sync::SyncArbiter;
use actix_web::fs::{NamedFile, StaticFileConfig, StaticFiles};
use actix_web::{http, middleware, middleware::cors::Cors, server, App, HttpRequest, Result};
use dotenv::dotenv;
use std::env;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub queue_handler: Addr<SongQueue>,
    pub db: Addr<DBExecutor>,
    pub radio: Addr<Radio>,
}

pub struct System;

/// Serves static files inside /static/client.
/// If requested path doesn't match any file then home page is returned.
fn serve_files(req: &HttpRequest<AppState>) -> Result<NamedFile> {
    let mut file_path = PathBuf::new();
    file_path.push("./static/client/");
    let tail: PathBuf = req.match_info().query("tail").unwrap();
    file_path.push(tail);
    let file_handler = NamedFile::open(&file_path);
    if !&file_path.is_dir() && file_handler.is_ok() {
        Ok(file_handler.unwrap())
    } else {
        Ok(NamedFile::open("./static/client/index.html")?)
    }
}

impl System {
    pub fn new() -> Self {
        ::std::env::set_var("RUST_LOG", "actix_web=info");
        env_logger::init();
        let sys = actix::System::new("home-fm-server");

        // start all of the needed actors and clone their addresses where they're needed
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let database_poll = new_pool(database_url).expect("Failed to create pool");
        let db = DBExecutor::new(database_poll.clone()).start();
        let second_db_addr = db.clone();
        let radio = Arbiter::start(|ctx| Radio::new());
        let io = SyncArbiter::start(1, move || MyIO { db: db.clone() });
        let queue_handler = SongQueue {
            IO: io.clone(),
            db: second_db_addr.clone(),
            songs_queue: Vec::new(),
            radio: radio.clone(),
            active_song: None,
        }
        .start();

        let app_state = AppState {
            queue_handler,
            db: second_db_addr.clone(),
            radio,
        };

        server::new(move || {
            App::with_state(app_state.clone())
                // add our resources (routes)
                .scope("/api", |scope| {
                    scope
                        .resource("/ws/", |r| r.route().f(ws_index))
                        .resource("/songs", |r| {
                            r.method(http::Method::GET).with(get_all_songs)
                        })
                        .resource("/songs/{id}", |r| {
                            r.method(http::Method::DELETE).with(delete_song)
                        })
                        .resource("/songs/{id}/{is_nsfw}", |r| {
                            r.method(http::Method::PUT).with(toggle_song_nsfw)
                        })
                        .resource("/config", |r| {
                            r.method(http::Method::PUT).with(update_config);
                            r.method(http::Method::GET).with(get_config);
                        })
                })
                .resource(r"/{tail:.*}", |r| {
                    r.method(http::Method::GET).f(serve_files)
                })
                // add middleware to log stuff
                .middleware(middleware::Logger::default())
                .middleware(Cors::build().finish())
        })
        .bind("127.0.0.1:8080")
        .unwrap()
        .start();;

        println!("Started http server: 127.0.0.1:8080");
        sys.run();
        System {}
    }
}
