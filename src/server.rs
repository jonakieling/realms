
use std::net::TcpStream;
use std::net::TcpListener;
use std::net::Shutdown;
use std::io::prelude::*;
use std::io;

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;

use chrono::{Local, DateTime};

use tui::layout::{Direction, Group, Size};
use tui::widgets::{List, Block, Borders, Item, Widget};
use tui::style::{Style, Color};

use tokens::*;

pub struct Universe {
	pub listener: TcpListener,
	pub realms: Vec<Realm>,
	pub requests: Vec<(usize, RealmsProtocol)>,
	pub clients: Vec<Client>
}

pub struct Client {
	id: usize,
	connected: bool,
	time: DateTime<Local>
}

impl Client {
	pub fn new(id: usize) -> Client {
		Client {
			id,
			connected: true,
			time: Local::now()
		}
	}
}

impl Universe {
	pub fn run(mut self, t: &mut Terminal<RawBackend>) -> Result<(), io::Error> {
	    draw_dashboard(t, &self.requests, &self.clients, &self.realms)?;
	    for stream in self.listener.incoming() {
			let mut stream = stream.unwrap();
			loop {
			    let mut buffer = [0; 512];

			    stream.read(&mut buffer).unwrap();
			    stream.flush().unwrap();


			    let (client_id, request): (usize, RealmsProtocol) = deserialize(&buffer).expect("could not deserialize client request.");

			    match request {
			        RealmsProtocol::Register => {
			        	let mut high = 0;
			        	for client in &self.clients {
			        	    high = client.id.max(high);
			        	}
			        	let mut id = self.clients.len().max(high);
						send_response(&RealmsProtocol::Connect(id), &stream)?;
						self.clients.push(Client::new(id));
			    		self.requests.push((id, request));
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
			    		for client in &mut self.clients {
			    		    if client.id == client_id {
			    		    	client.connected = false;
			    		    }
			    		}
			    		// draw dashboard update before client exits
			    		draw_dashboard(t, &self.requests, &self.clients, &self.realms)?;
			        	break;
			        },
			        _ => { }
			    }
			    draw_dashboard(t, &self.requests, &self.clients, &self.realms)?;
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


fn draw_dashboard(t: &mut Terminal<RawBackend>, requests: &Vec<(usize, RealmsProtocol)>, clients: &Vec<Client>, realms: &Vec<Realm>) -> Result<(), io::Error> {
	let t_size = t.size().unwrap();

	Group::default()
        .direction(Direction::Horizontal)
		.sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, &t_size, |t, chunks| {
            let style = Style::default();
            let highlight = Style::default().fg(Color::Yellow);

        	let requests = requests.iter().rev().map(|(client, request)| {
        		match request {
        		    RealmsProtocol::Register | RealmsProtocol::Connect(_) | RealmsProtocol::Quit => Item::StyledData(
                    format!("{} {}", client, request),
	                    &highlight
                	),
        		    _ => Item::StyledData(
                    format!("{} {}", client, request),
	                    &style
	                ),
        		}
            });
        	let clients = clients.iter().rev().map(|client| {
        		match client.connected {
        		    true => Item::StyledData(
	                    format!("{} {}", client.id, client.time.format("%H:%M:%S %d.%m.%y")),
	                    &highlight
                	),
        		    false => Item::StyledData(
	                    format!("{} {}", client.id, client.time.format("%H:%M:%S %d.%m.%y")),
	                    &style
	                ),
        		}
            });
        	let realms = realms.iter().rev().map(|realm| {
        		Item::StyledData(
                    format!("{}", realm.id),
                    &style
                )
            });

    		List::new(requests)
                .block(Block::default().borders(Borders::ALL).title("Requests"))
                .render(t, &chunks[0]);


			Group::default()
		        .direction(Direction::Vertical)
				.sizes(&[Size::Percent(50), Size::Percent(50)])
		        .render(t, &chunks[1], |t, chunks| {

		    		List::new(clients)
		                .block(Block::default().borders(Borders::ALL).title("Clients"))
		                .render(t, &chunks[0]);

		    		List::new(realms)
		                .block(Block::default().borders(Borders::ALL).title("Realms"))
		                .render(t, &chunks[1]);
		        });
        });
	t.draw()
}