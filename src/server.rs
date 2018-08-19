
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
	pub requests: Vec<(usize, RealmsProtocol)>,
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


			    let (client_id, request): (usize, RealmsProtocol) = deserialize(&buffer).expect("could not deserialize client request.");

			    match request {
			        RealmsProtocol::Register => {
			        	let mut new_id = self.clients.len();
			        	if let Some(current_highest) = self.clients.iter().max() {
			        	    new_id = (current_highest + 1).max(new_id);
			        	}
						send_response(&RealmsProtocol::Connect(new_id), &stream)?;
						self.clients.push(new_id);
			    		self.requests.push((client_id, request));
			        },
			        RealmsProtocol::RequestRealmsList => {
		        		let realms = self.realms.iter().map(|realm| {
		        			realm.id
		        		}).collect();
						send_response(&RealmsProtocol::RealmsList(realms), &stream)?;
			    		self.requests.push((client_id, request));
			        },
			        RealmsProtocol::RequestNewRealm => {
			        	let id = self.realms.len();
			        	let realm = Realm {
			    			island: Island::new(),
			    			expedition: Expedition::new(),
			    			id,
			    			age: 0
			    		};
						send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
			    		self.realms.push(realm);
			    		self.requests.push((client_id, request));
			        },
			        RealmsProtocol::RequestRealm(realm_id) => {
			        	let realm;
			        	if self.realms.len() > realm_id {
			        	    realm = self.realms[realm_id].clone();
							send_response(&RealmsProtocol::Realm(realm), &stream)?;
			        	} else {
			        		// send new realm on miss
				        	let id = self.realms.len();
				        	realm = Realm {
				    			island: Island::new(),
				    			expedition: Expedition::new(),
				    			id,
				    			age: 0
				    		};
							send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
				    		self.realms.push(realm);
			        	}
			    		self.requests.push((client_id, request));
			        },
			        RealmsProtocol::Quit => {
						send_response(&RealmsProtocol::Quit, &stream)?;
						stream.shutdown(Shutdown::Both).expect("stream could not shut down.");
			    		self.requests.push((client_id, request));
			    		// draw dashboard update before client exits
			    		draw_dashboard(t, &self.requests, &self.clients)?;
			        	break;
			        },
			        _ => { }
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


fn draw_dashboard(t: &mut Terminal<RawBackend>, requests: &Vec<(usize, RealmsProtocol)>, clients: &Vec<usize>) -> Result<(), io::Error> {
	let t_size = t.size().unwrap();

	Group::default()
        .direction(Direction::Horizontal)
		.sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, &t_size, |t, chunks| {
            let style = Style::default();
        	let requests = requests.iter().rev().map(|(client, request)| {
                Item::StyledData(
                    format!("{} {}", client, request),
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