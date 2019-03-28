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

// TODO: move songs_queue into radio, so song playing logic and managing queue will be done independently in radio actor

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
        let second_db_addr = db.clone();
        // how can I simplify this, so I won't run into borrowing problems after db move into the closure?
        let io = SyncArbiter::start(num_cpus::get(), move || MyIO { db: db.clone() });

        let radio = SyncArbiter::start(num_cpus::get(), move || Radio {
            IO: io.clone(),
            db: second_db_addr.clone(),
            songs_queue: Vec::new(),
        });

        let mut server = server::new(|| {
            App::new() // <- create app with shared state
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
