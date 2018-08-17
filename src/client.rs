
use std::net::{TcpStream, Shutdown};
use std::sync::mpsc::Receiver;
use std::io::prelude::*;
use std::io;

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;
use termion::event;

use tui::layout::{Direction, Group, Size};
use tui::widgets::{Row, Table, Widget, Paragraph, Block, Borders};
use tui::style::{Style, Color};

use Event;
use tokens::*;

pub struct Periscope {
	pub stream: TcpStream,
	pub realm: Option<Realm>,
	pub id: usize
}

impl Periscope {
	pub fn new(stream: TcpStream) -> Periscope {
		let mut periscope = Periscope {
			stream,
			realm: None,
			id: 0
		};

		periscope.send_request(RealmsProtocol::CONNECT(None));

		periscope
	}

	pub fn run(mut self, t: &mut Terminal<RawBackend>, rx: &Receiver<Event>) -> Result<(), io::Error> {
		loop {
			let t_size = t.size().unwrap();
			
			Group::default()
		        .direction(Direction::Vertical)
        		.sizes(&[Size::Fixed(6), Size::Min(0)])
		        .render(t, &t_size, |t, main_chunks| {

				Group::default()
			        .direction(Direction::Horizontal)
	        		.sizes(&[Size::Percent(75), Size::Percent(25)])
			        .render(t, &main_chunks[0], |t, chunks| {
			        	Paragraph::default()
					        .text(
					            "request new realm with {mod=bold r}\nchange island with {mod=bold i}\nchange expedition with {mod=bold e}\nexit with {mod=bold q}",
					        ).block(Block::default().title("Abstract").borders(Borders::ALL))
					        .render(t, &chunks[0]);


			        	Paragraph::default()
					        .text(
					            &format!("id {{mod=bold {}}}", self.id),
					        ).block(Block::default().title("Client").borders(Borders::ALL))
					        .render(t, &chunks[1]);

			        });

			        if let Some(ref realm) = self.realm {

						Group::default()
					        .direction(Direction::Horizontal)
			        		.sizes(&[Size::Percent(50), Size::Percent(50)])
					        .render(t, &main_chunks[1], |t, chunks| {
					            let style = Style::default();

					            Table::new(
					                ["terrain", "particularities"].into_iter(),
					                realm.island.tiles.iter().map(|tile| {
					                    Row::StyledData(vec![format!("{}", tile), format!("{:?}", tile.particularities)].into_iter(), &style)
					                })
					            ).block(Block::default().title("Island").borders(Borders::ALL))
				                .header_style(Style::default().fg(Color::Yellow))
				                .widths(&[13, 12])
				                .render(t, &chunks[0]);

					            Table::new(
					                ["explorer"].into_iter(),
					                realm.expedition.explorers.iter().map(|explorer| {
					                    Row::StyledData(vec![format!("{:?}", &explorer)].into_iter(), &style)
					                })
					            ).block(Block::default().title("Expedition").borders(Borders::ALL))
				                .header_style(Style::default().fg(Color::Yellow))
				                .widths(&[30])
				                .render(t, &chunks[1]);
					        });
			        }
		        });
	        t.draw()?;

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
			    		event::Key::Char('q') => {
		    				self.send_request(RealmsProtocol::QUIT);
		    				break;
			    		},
			    		_ => { }
			    	}
			     }
			}
		}
	    t.show_cursor().unwrap();
	    t.clear().unwrap();

	    Ok(())
	}

	pub fn send_request(&mut self, request: RealmsProtocol) {
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
	        RealmsProtocol::CONNECT(Some(id)) => {
	    		self.id = id;
	        },
	        RealmsProtocol::REALM(Some(realm)) => {
	    		let realm: Realm = deserialize(&realm).expect("could not deserialize realm");
	    		self.realm = Some(realm);
	        },
	        RealmsProtocol::ISLAND(Some(island)) => {
	    		let island: Island = deserialize(&island).expect("could not deserialize island");
	    		if let Some(ref mut realm) = self.realm {
	    			realm.island = island;
	    		}
	        },
	        RealmsProtocol::EXPEDITION(Some(expedition)) => {
	    		let expedition: Expedition = deserialize(&expedition).expect("could not deserialize expedition");

	    		if let Some(ref mut realm) = self.realm {
	    			realm.expedition = expedition;
	    		}
	        },
	        RealmsProtocol::STATE(Some(state)) => {
	    		let _state: String = deserialize(&state).expect("could not deserialize state");
	        },
	        RealmsProtocol::QUIT => {
				self.stream.shutdown(Shutdown::Both).expect("connection should have terminated.");
	        },
	        _ => {
	        }
	    }
	}
}