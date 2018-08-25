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
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time;
use std::net::TcpStream;
use std::net::TcpListener;

use tui::Terminal;
use tui::backend::RawBackend;
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


	// uberspace: j0na.net:64245
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

    // tui terminal
    let backend = RawBackend::new().unwrap();
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();
    terminal.hide_cursor().unwrap();

	match mode {
	    Mode::Client => {

			// inter process communication
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
		 		periscope.run(&mut terminal, &rx).expect("io error");
			}
	    },
	    Mode::Server => {
			let listener = TcpListener::bind(host).expect(&format!("could not bind tcp listener to {}", host));
    		let universe = server::Universe { listener, realms: vec![], requests: vec![], clients: HashMap::new() };
	    	universe.run(&mut terminal).expect("io error");
	    }
	}
}
