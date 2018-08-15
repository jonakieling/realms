
use std::collections::HashMap;
use std::net::TcpListener;
use std::net::Shutdown;
use std::io::prelude::*;
use std::io;

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;

use tokens::*;

pub struct Universe {
	pub listener: TcpListener,
	pub realms: HashMap<&'static str, Realm>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RealmsProtocol {
	ISLAND(Option<Vec<u8>>),
	EXPEDITION(Option<Vec<u8>>),
	STATE(Option<Vec<u8>>),
	QUIT
}

impl Universe {
	pub fn run(self, _t: &mut Terminal<RawBackend>) -> Result<(), io::Error> {
	    for stream in self.listener.incoming() {
			let mut stream = stream.unwrap();
			loop {
			    let mut buffer = [0; 512];

			    stream.read(&mut buffer).unwrap();
			    stream.flush().unwrap();


			    let request: RealmsProtocol = deserialize(&buffer).expect("could not deserialize client request");

			    match request {
			        RealmsProtocol::ISLAND(_) => {
			    		let island_bytes = serialize(&Island::new()).expect("could not serialize island");
			    		let data = serialize(&RealmsProtocol::ISLAND(Some(island_bytes))).expect("could not serialize data package for island response.");
						stream.write(&data).expect("could not write to tcp stream.");
						stream.flush().unwrap();
			        },
			        RealmsProtocol::EXPEDITION(_) => {
			    		let expedition_bytes = serialize(&Expedition::new()).expect("could not serialize expedition");
			    		let data = serialize(&RealmsProtocol::EXPEDITION(Some(expedition_bytes))).expect("could not serialize data package for expedition response.");
						stream.write(&data).expect("could not write to tcp stream.");
						stream.flush().unwrap();
			        },
			        RealmsProtocol::STATE(_) => {
			    		let state_bytes = serialize(&state()).expect("could not serialize state");
			    		let data = serialize(&RealmsProtocol::STATE(Some(state_bytes))).expect("could not serialize data package for state response.");
						stream.write(&data).expect("could not write to tcp stream.");
						stream.flush().unwrap();
			        },
			        RealmsProtocol::QUIT => {
			    		let data = serialize(&RealmsProtocol::QUIT).expect("could not serialize data package for quit response.");
						stream.write(&data).expect("could not write to tcp stream.");
						stream.flush().unwrap();
						stream.shutdown(Shutdown::Both).expect("connection should have terminated.");
			        	break;
			        }
			    }
			}
	    }

	    Ok(())
	}
}

fn state() -> String {
	"state not implemented yet.".to_string()
}