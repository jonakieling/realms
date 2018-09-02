#[macro_use]
extern crate serde_derive;
extern crate serde;

extern crate bincode;

extern crate rand;

extern crate tui;
extern crate termion;

extern crate chrono;

extern crate itertools;

extern crate uuid;

use std::env;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time;
use std::net::TcpStream;

use termion::event;
use termion::input::TermRead;

mod client;
mod client_dashboard;
mod server;
mod server_dashboard;
mod tokens;
mod utility;
mod realms;
mod hex;

#[derive(Debug)]
pub enum Mode {
    Server,
    Client
}

pub enum Event {
    Input(event::Key),
    Tick,
}

fn main() {
	let args: Vec<String> = env::args().collect();

	let mut mode = Mode::Client;
	let mut host = "127.0.0.1:8080";
	if args.len() >= 2 && &args[1] == "server" {
    	mode = Mode::Server;
	}
	if args.len() >= 2 && &args[1] == "client" {
    	mode = Mode::Client;
	}
	if args.len() >= 3 {
    	host = &args[2];
	}

	match mode {
	    Mode::Client => {

		    let (tx, rx) = mpsc::channel();
			let input_tx = tx.clone();

			// event loop
			thread::spawn(move || {
		        let stdin = io::stdin();
		        for c in stdin.keys() {
		            let evt = c.unwrap();
		            input_tx.send(Event::Input(evt)).unwrap();
		            if evt == event::Key::Char('q') {
		                break;
		            }
		        }
		    });

			// tick loop
		    thread::spawn(move || {
		        let tx = tx.clone();
		        loop {
		            tx.send(Event::Tick).unwrap();
		            thread::sleep(time::Duration::from_millis(500));
		        }
			});

	    	if let Ok(stream) = TcpStream::connect(host) {
	    		let periscope = client::Periscope::new(stream);
		 		periscope.run(&rx).expect("io error");
			}
	    },
	    Mode::Server => {
	    	server::run(host.to_string());
	    }
	}
}
