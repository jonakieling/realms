
use std::net::TcpStream;
use std::net::TcpListener;
use std::net::Shutdown;
use std::io::prelude::*;
use std::io;

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;

use tui::layout::{Direction, Group, Size};
use tui::widgets::{List, Block, Borders, Item, Widget};
use tui::style::{Style};

use tokens::*;

pub struct Universe {
	pub listener: TcpListener,
	pub realms: Vec<Realm>,
	pub requests: Vec<RealmsProtocol>,
	pub clients: Vec<usize>
}

impl Universe {
	pub fn run(mut self, t: &mut Terminal<RawBackend>) -> Result<(), io::Error> {
	    draw_dashboard(t, &self.requests, &self.clients)?;
	    for stream in self.listener.incoming() {
			let mut stream = stream.unwrap();
			loop {
			    let mut buffer = [0; 512];

			    stream.read(&mut buffer).unwrap();
			    stream.flush().unwrap();


			    let request: RealmsProtocol = deserialize(&buffer).expect("could not deserialize client request.");

			    match request {
			        RealmsProtocol::CONNECT(_) => {
			        	let mut new_id = self.clients.len();
			        	if let Some(current_highest) = self.clients.iter().max() {
			        	    new_id = (current_highest + 1).max(new_id);
			        	}
						send_response(&RealmsProtocol::CONNECT(Some(new_id)), &stream)?;
						self.clients.push(new_id);
			    		self.requests.push(request);
			        },
			        RealmsProtocol::REALM(_) => {
			        	let id = self.realms.len();
			        	let realm = Realm {
			    			island: Island::new(),
			    			expedition: Expedition::new(),
			    			id,
			    			age: 0
			    		};
			    		let realm_bytes = serialize(&realm).expect("could not serialize realm.");
						send_response(&RealmsProtocol::REALM(Some(realm_bytes)), &stream)?;
			    		self.realms.push(realm);
			    		self.requests.push(request);
			        },
			        RealmsProtocol::ISLAND(_) => {
			    		let island_bytes = serialize(&Island::new()).expect("could not serialize island.");
						send_response(&RealmsProtocol::ISLAND(Some(island_bytes)), &stream)?;
			    		self.requests.push(request);
			        },
			        RealmsProtocol::EXPEDITION(_) => {
			    		let expedition_bytes = serialize(&Expedition::new()).expect("could not serialize expedition.");
						send_response(&RealmsProtocol::EXPEDITION(Some(expedition_bytes)), &stream)?;
			    		self.requests.push(request);
			        },
			        RealmsProtocol::STATE(_) => {
			    		let state_bytes = serialize(&state()).expect("could not serialize state.");
						send_response(&RealmsProtocol::STATE(Some(state_bytes)), &stream)?;
			        },
			        RealmsProtocol::QUIT => {
						send_response(&RealmsProtocol::QUIT, &stream)?;
						stream.shutdown(Shutdown::Both).expect("stream could not shut down.");
			    		self.requests.push(request);
			    		// draw dashboard update before client exits
			    		draw_dashboard(t, &self.requests, &self.clients)?;
			        	break;
			        }
			    }
			    draw_dashboard(t, &self.requests, &self.clients)?;
			}
	    }
	    t.show_cursor().unwrap();
	    t.clear().unwrap();

	    Ok(())
	}
}

fn send_response(data: &RealmsProtocol, mut stream: &TcpStream) -> Result<(), io::Error> {
	let raw = serialize(data).expect("could not serialize data for response.");
	stream.write(&raw).expect("could not write to tcp stream.");
	stream.flush().unwrap();
	Ok(())
}


fn draw_dashboard(t: &mut Terminal<RawBackend>, requests: &Vec<RealmsProtocol>, clients: &Vec<usize>) -> Result<(), io::Error> {
	let t_size = t.size().unwrap();

	Group::default()
        .direction(Direction::Horizontal)
		.sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, &t_size, |t, chunks| {
            let style = Style::default();
        	let requests = requests.iter().rev().map(|request| {
                Item::StyledData(
                    format!("{}", request),
                    &style
                )
            });
        	let clients = clients.iter().rev().map(|client| {
                Item::StyledData(
                    format!("{}", client),
                    &style
                )
            });

    		List::new(requests)
                .block(Block::default().borders(Borders::ALL).title("Realms Requests"))
                .render(t, &chunks[0]);

    		List::new(clients)
                .block(Block::default().borders(Borders::ALL).title("Realms Clients"))
                .render(t, &chunks[1]);
        });
	t.draw()
}

fn state() -> String {
	"state not implemented yet.".to_string()
}