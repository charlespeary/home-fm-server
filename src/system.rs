use super::io::MyIO;
use super::radio::Radio;
use super::song::Song;
use super::web_socket::ws_index;
use crate::db::DBExecutor;
use actix::prelude::*;
use actix::sync::SyncArbiter;
use actix::Addr;
use actix_web::{middleware, server, App};
use listenfd::ListenFd;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub current_song: Arc<Mutex<String>>,
    pub IO: Addr<MyIO>,
    pub songs_queue: Arc<Mutex<Vec<Song>>>,
    pub radio: Addr<Radio>,
    pub db: Addr<DBExecutor>,
}

pub struct System {}

impl System {
    pub fn new() -> Self {
        ::std::env::set_var("RUST_LOG", "actix_web=info");
        env_logger::init();
        let sys = actix::System::new("home-fm-server");
        // listenfd object that brings hotreloading
        let mut listenfd = ListenFd::from_env();

        // Initial state of the app filled with Arc<Mutex>> to make it shareable between states

        // start all of the needed actors and clone their addresses where they're needed
        let db = SyncArbiter::start(num_cpus::get(), move || DBExecutor::new());
        // how can I simplify this, so I won't run into borrowing problems after db move into the closure?
        let db_addr = db.clone();
        let io = SyncArbiter::start(num_cpus::get(), move || MyIO { db: db.clone() });
        let radio = Radio { IO: io.clone() }.start();

        let state = AppState {
            current_song: Arc::new(Mutex::new(String::new())),
            IO: io.clone(),
            songs_queue: Arc::new(Mutex::new(Vec::new())),
            radio: radio.clone(),
            db: db_addr.clone(),
        };

        let mut server = server::new(move || {
            App::with_state(state.clone()) // <- create app with shared state
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
