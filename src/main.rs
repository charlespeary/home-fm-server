#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate listenfd;
extern crate serde;
#[macro_use]
extern crate failure;
mod app_state;
mod system;
use system::System;

fn main() {
    let system = System::new();
}
