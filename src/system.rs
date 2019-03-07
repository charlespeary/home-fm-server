use super::app_state::{get_app_state, set_current_song, AppState};
use actix_web::{http::Method, middleware, server, App};
use listenfd::ListenFd;
use std::sync::{Arc, Mutex};

pub struct System {}

impl System {
    pub fn new() -> Self {
        ::std::env::set_var("RUST_LOG", "actix_web=info");
        env_logger::init();
        let sys = actix::System::new("ws-example");
        // listenfd object that brings hotreloading
        let mut listenfd = ListenFd::from_env();
        //move is necessary to give closure below ownership of counter
        let mut server = server::new(move || {
            App::with_state(AppState {
                current_song: Arc::new(Mutex::new(String::from("DESPACITO"))),
            }) // <- create app with shared state
            // add our resources (routes)
            .resource("/song", |r| r.method(Method::GET).f(get_app_state))
            .resource("/song/set_current", |r| {
                r.method(Method::POST).with(set_current_song)
            })
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
