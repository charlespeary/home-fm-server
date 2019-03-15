#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[macro_use]
mod song;
mod io;
mod system;
mod web_socket;
use system::System;
extern crate num_cpus;

fn main() {
    let system = System::new();
}
