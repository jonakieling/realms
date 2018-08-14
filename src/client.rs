
use std::io::prelude::*;
use std::net::TcpStream;

use rand::{thread_rng, distributions::Uniform, Rng};

use bincode::{serialize};

use tokens::{Island, Tile, Terrain, Particularity};

pub fn run() {
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

	let island = Island {
		tiles
	};

	if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {
		let msg = serialize(&island).expect("could not serialize island");
 		stream.write(&msg).expect("could not write to tcp stream.");
	}
}
