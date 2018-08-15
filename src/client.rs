
use std::net::{TcpStream, Shutdown};
use std::sync::mpsc::Receiver;
use std::io::prelude::*;
use std::io;

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;
use termion::event;

use Event;
use server::RealmsProtocol;
use tokens::*;

pub struct Periscope {
	pub stream: TcpStream,
	pub realm: Option<Realm>
}

impl Periscope {
	pub fn run(mut self, _t: &mut Terminal<RawBackend>, rx: &Receiver<Event>) -> Result<(), io::Error> {
		loop {
			let evt = rx.recv().unwrap();
			match evt {
			    Event::Tick => {
    				self.send_request(RealmsProtocol::STATE(None));
			    },
			    Event::Input(key) => {
			    	match key {
			    		event::Key::Char('r') => {
		    				self.send_request(RealmsProtocol::REALM(None));
			    		},
			    		event::Key::Char('i') => {
		    				self.send_request(RealmsProtocol::ISLAND(None));
			    		},
			    		event::Key::Char('e') => {
		    				self.send_request(RealmsProtocol::EXPEDITION(None));
			    		},
			    		event::Key::Char('c') => {
		    				println!("{:?}", self.realm);
			    		},
			    		event::Key::Char('q') => {
		    				self.send_request(RealmsProtocol::QUIT);
		    				break;
			    		},
			    		_ => { }
			    	}
			     }
			}
		}

	    Ok(())
	}

	fn send_request(&mut self, request: RealmsProtocol) {
		let data = serialize(&request).expect("could not serialize data package for request.");
		self.stream.write(&data).expect("could not write to tcp stream.");
		self.stream.flush().unwrap();
		self.handle_response();
	}

	fn handle_response(&mut self) {
		let mut buffer = [0; 512];

	    self.stream.read(&mut buffer).unwrap();
	    self.stream.flush().unwrap();


	    let response: RealmsProtocol = deserialize(&buffer).expect("could not deserialize server response");

	    match response {
	        RealmsProtocol::REALM(Some(realm)) => {
	    		let realm: Realm = deserialize(&realm).expect("could not deserialize realm");
	    		self.realm = Some(realm);
	        },
	        RealmsProtocol::ISLAND(Some(island)) => {
	    		let _island: Island = deserialize(&island).expect("could not deserialize island");
	        },
	        RealmsProtocol::EXPEDITION(Some(expedition)) => {
	    		let _expedition: Expedition = deserialize(&expedition).expect("could not deserialize expedition");
	        },
	        RealmsProtocol::STATE(Some(state)) => {
	    		let _state: String = deserialize(&state).expect("could not deserialize state");
	        },
	        RealmsProtocol::QUIT => {
				self.stream.shutdown(Shutdown::Both).expect("connection should have terminated.");
				println!("quitting");
	        },
	        _ => {
	        	println!("server response not recognized.");
	        }
	    }
	}
}