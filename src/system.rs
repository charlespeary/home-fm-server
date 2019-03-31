use super::io::MyIO;
use super::radio::Radio;
use super::song_queue::SongQueue;
use super::web_socket::ws_index;
use crate::db::{new_pool, DBExecutor};
use actix::prelude::*;
use actix::sync::SyncArbiter;
use actix_web::{middleware, server, App};
use dotenv::dotenv;
use listenfd::ListenFd;
use std::env;

#[derive(Clone)]
pub struct AppState {
    pub queue_handler: Addr<SongQueue>,
}

pub struct System {}

impl System {
    pub fn new() -> Self {
        ::std::env::set_var("RUST_LOG", "actix_web=info");
        env_logger::init();
        let sys = actix::System::new("home-fm-server");
        // listenfd object that brings hotreloading
        let mut listenfd = ListenFd::from_env();

        // start all of the needed actors and clone their addresses where they're needed
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let database_poll = new_pool(database_url).expect("Failed to create pool");

        //        let db = SyncArbiter::start(num_cpus::get(), move || {
        //            DBExecutor::new(database_poll.clone())
        //        });
        let db = DBExecutor::new(database_poll.clone()).start();
        let second_db_addr = db.clone();
        // how can I simplify this, so I won't run into borrowing problems after db move into the closure?
        let io = SyncArbiter::start(num_cpus::get(), move || MyIO { db: db.clone() });
        let radio = Radio {}.start();
        let queue_handler = SongQueue {
            IO: io.clone(),
            db: second_db_addr.clone(),
            songs_queue: Vec::new(),
            radio: radio.clone(),
        }
        .start();

        let app_state = AppState { queue_handler };

        let mut server = server::new(move || {
            App::with_state(app_state.clone()) // <- create app with shared state
                // add our resources (routes)
                .resource("/ws/", |r| r.route().f(ws_index))
                // add middleware to log stuff
                .middleware(middleware::Logger::default())
        });

        server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
            server.listen(l)
        } else {
            server.bind("127.0.0.1:8080").unwrap()
        };
        println!("Started http server: 127.0.0.1:8080");
        server.run();
        System {}
    }
}
