#[macro_use]
extern crate serde_derive;
extern crate serde;

extern crate bincode;

extern crate rand;

extern crate tui;
extern crate termion;

extern crate chrono;

use std::collections::HashMap;
use std::env;
use std::io;
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
mod server;
mod tokens;
mod utility;

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
	if args.len() == 2 && &args[1] == "server" {
    	mode = Mode::Server;
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

	    	if let Ok(stream) = TcpStream::connect("127.0.0.1:8080") {
	    		let periscope = client::Periscope::new(stream);
		 		periscope.run(&mut terminal, &rx).expect("io error");
			}
	    },
	    Mode::Server => {
			let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    		let universe = server::Universe { listener, realms: vec![], requests: vec![], clients: vec![], client_locations: HashMap::new() };
	    	universe.run(&mut terminal).expect("io error");
	    }
	}
}
