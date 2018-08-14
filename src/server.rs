
use std::io::prelude::*;
use std::net::TcpListener;

use bincode::{deserialize};

use tokens::Island;

pub fn run() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

	    let mut buffer = [0; 512];

	    stream.read(&mut buffer).unwrap();


		let island: Island = deserialize(&buffer).expect("could not serialize island");

	    println!("{:?}", island);
    }
}
