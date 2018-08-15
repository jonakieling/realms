
use std::net::{TcpStream, Shutdown};
use std::sync::mpsc::Receiver;
use std::io::prelude::*;
use std::io;

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;
use termion::event;

use Event;
use ClientApp;
use server::RealmsProtocol;
use tokens::*;

impl ClientApp {
	pub fn run(self, _t: &mut Terminal<RawBackend>, rx: &Receiver<Event>) -> Result<(), io::Error> {
		loop {
			let evt = rx.recv().unwrap();
			match evt {
			    Event::Tick => {
			    	pull_state(&self.stream);
			    },
			    Event::Input(key) => {
			    	match key {
			    		event::Key::Char('i') => {
		    				request_island(&self.stream);
			    		},
			    		event::Key::Char('e') => {
		    				request_expedition(&self.stream);
			    		},
			    		event::Key::Char('q') => {
		    				quit(&self.stream);
		    				break;
			    		},
			    		_ => { }
			    	}
			     }
			}
		}

	    Ok(())
	}
}

fn handle_response(mut stream: &TcpStream) {
	let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();
    stream.flush().unwrap();


    let response: RealmsProtocol = deserialize(&buffer).expect("could not deserialize server response");

    match response {
        RealmsProtocol::ISLAND(Some(island)) => {
    		let island: Island = deserialize(&island).expect("could not deserialize island");
			println!("{:?}", island);
        },
        RealmsProtocol::EXPEDITION(Some(expedition)) => {
    		let expedition: Expedition = deserialize(&expedition).expect("could not deserialize expedition");
			println!("{:?}", expedition);
        },
        RealmsProtocol::STATE(Some(state)) => {
    		let state: String = deserialize(&state).expect("could not deserialize state");
			println!("{:?}", state);
        },
        RealmsProtocol::QUIT => {
			stream.shutdown(Shutdown::Both).expect("connection should have terminated.");
			println!("quitting");
        },
        _ => {
        	println!("server response not recognized.");
        }
    }
}

pub fn request_island(mut stream: &TcpStream) {
	let data = serialize(&RealmsProtocol::ISLAND(None)).expect("could not serialize data package for island request.");
	stream.write(&data).expect("could not write to tcp stream.");
	stream.flush().unwrap();
	handle_response(stream);
}

pub fn request_expedition(mut stream: &TcpStream) {
	let data = serialize(&RealmsProtocol::EXPEDITION(None)).expect("could not serialize data package for expedition request.");
	stream.write(&data).expect("could not write to tcp stream.");
	stream.flush().unwrap();
	handle_response(stream);
}

pub fn quit(mut stream: &TcpStream) {
	let data = serialize(&RealmsProtocol::QUIT).expect("could not serialize data package for quit request.");
	stream.write(&data).expect("could not write to tcp stream.");
	stream.flush().unwrap();
	handle_response(stream);
}

pub fn pull_state(mut stream: &TcpStream) {
	let data = serialize(&RealmsProtocol::STATE(None)).expect("could not serialize data package for state request.");
	stream.write(&data).expect("could not write to tcp stream.");
	stream.flush().unwrap();
	handle_response(stream);
}