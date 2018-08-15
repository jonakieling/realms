
use std::sync::mpsc::Receiver;
use std::net::Shutdown;
use std::io::prelude::*;
use std::io;

use rand::{thread_rng, distributions::Uniform, Rng};

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;

use ServerApp;
use tokens::*;
use Event;

#[derive(Serialize, Deserialize, Debug)]
pub enum RealmsProtocol {
	ISLAND(Option<Vec<u8>>),
	EXPEDITION(Option<Vec<u8>>),
	STATE(Option<Vec<u8>>),
	QUIT
}

impl ServerApp {
	pub fn run(self, _t: &mut Terminal<RawBackend>, _rx: &Receiver<Event>) -> Result<(), io::Error> {
	    for stream in self.listener.incoming() {
			let mut stream = stream.unwrap();
			loop {
			    let mut buffer = [0; 512];

			    stream.read(&mut buffer).unwrap();
			    stream.flush().unwrap();


			    let request: RealmsProtocol = deserialize(&buffer).expect("could not deserialize client request");

			    match request {
			        RealmsProtocol::ISLAND(_) => {
			    		let island_bytes = serialize(&island()).expect("could not serialize island");
			    		let data = serialize(&RealmsProtocol::ISLAND(Some(island_bytes))).expect("could not serialize data package for island response.");
						stream.write(&data).expect("could not write to tcp stream.");
						stream.flush().unwrap();
			        },
			        RealmsProtocol::EXPEDITION(_) => {
			    		let expedition_bytes = serialize(&expedition()).expect("could not serialize expedition");
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

fn island() -> Island {
	let mut rng = thread_rng();
	let mut rng2 = thread_rng();
	let tiles: Vec<Tile> = rng.sample_iter(&Uniform::new_inclusive(1, 4)).take(19).map(|number| {
		let terrain = match number {
		    1 => Terrain::Coast,
		    2 => Terrain::Planes,
		    3 => Terrain::Forest,
		    _ => Terrain::Mountain,
		};

		let how_many_particularities = rng2.sample(&Uniform::new_inclusive(0, 1));

		let particularities: Vec<Particularity> = rng2.sample_iter(&Uniform::new_inclusive(1, 3)).take(how_many_particularities).map(|number| {
			match number {
			    1 => Particularity::Town,
			    2 => Particularity::River,
			    _ => Particularity::Carravan
			}
		}).collect();

		Tile {
			terrain,
			particularities
		}
	}).collect();

	Island {
		tiles
	}
}

fn expedition() -> Expedition {
	Expedition {
		explorers: vec![],
		gear: vec![]
	}
}

fn state() -> String {
	"state not implemented yet.".to_string()
}