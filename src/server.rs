
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
	    draw_dashboard(t, &self.requests)?;
	    for stream in self.listener.incoming() {
			let mut stream = stream.unwrap();
			loop {
			    let mut buffer = [0; 512];

			    stream.read(&mut buffer).unwrap();
			    stream.flush().unwrap();


			    let request: RealmsProtocol = deserialize(&buffer).expect("could not deserialize client request");

			    match request {
			        RealmsProtocol::CONNECT(_) => {
			        	let mut new_id = self.clients.len();
			        	if let Some(current_highest) = self.clients.iter().max() {
			        	    new_id = (current_highest + 1).max(new_id);
			        	}
			    		let data = serialize(&RealmsProtocol::CONNECT(Some(new_id))).expect("could not serialize data package for connect response.");
						stream.write(&data).expect("could not write to tcp stream.");
						self.clients.push(new_id);
						stream.flush().unwrap();
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
			    		let realm_bytes = serialize(&realm).expect("could not serialize realm");
			    		let data = serialize(&RealmsProtocol::REALM(Some(realm_bytes))).expect("could not serialize data package for realm response.");
						stream.write(&data).expect("could not write to tcp stream.");
			    		self.realms.push(realm);
						stream.flush().unwrap();
			    		self.requests.push(request);
			        },
			        RealmsProtocol::ISLAND(_) => {
			    		let island_bytes = serialize(&Island::new()).expect("could not serialize island");
			    		let data = serialize(&RealmsProtocol::ISLAND(Some(island_bytes))).expect("could not serialize data package for island response.");
						stream.write(&data).expect("could not write to tcp stream.");
						stream.flush().unwrap();
			    		self.requests.push(request);
			        },
			        RealmsProtocol::EXPEDITION(_) => {
			    		let expedition_bytes = serialize(&Expedition::new()).expect("could not serialize expedition");
			    		let data = serialize(&RealmsProtocol::EXPEDITION(Some(expedition_bytes))).expect("could not serialize data package for expedition response.");
						stream.write(&data).expect("could not write to tcp stream.");
						stream.flush().unwrap();
			    		self.requests.push(request);
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
			    		self.requests.push(request);
			    		// draw dashboard update befor client exits
			    		draw_dashboard(t, &self.requests)?;
			        	break;
			        }
			    }
			    draw_dashboard(t, &self.requests)?;
			}
	    }
	    t.show_cursor().unwrap();
	    t.clear().unwrap();

	    Ok(())
	}
}


fn draw_dashboard(t: &mut Terminal<RawBackend>, requests: &Vec<RealmsProtocol>) -> Result<(), io::Error> {
	let t_size = t.size().unwrap();

	Group::default()
        .direction(Direction::Vertical)
		.sizes(&[Size::Min(0)])
        .render(t, &t_size, |t, chunks| {
            let style = Style::default();
        	let requests = requests.iter().rev().map(|ref mut request| {
                    Item::StyledData(
                        format!("{}", request),
                        &style
                    )
                });

    		List::new(requests)
                .block(Block::default().borders(Borders::ALL).title("Realms Server"))
                .render(t, &chunks[0]);
        });
	t.draw()
}

fn state() -> String {
	"state not implemented yet.".to_string()
}