#[macro_use]
extern crate serde_derive;
extern crate serde;

extern crate bincode;

extern crate rand;

use std::env;

mod client;
mod server;
mod tokens;

#[derive(Debug)]
enum Mode {
    Server,
    Client
}

fn main() {
	let args: Vec<String> = env::args().collect();

	let mut mode = Mode::Client;
	if args.len() == 2 && &args[1] == "server"{
    	mode = Mode::Server;
	}

	match mode {
	    Mode::Client => client::run(),
	    Mode::Server => server::run(),
	}
}
