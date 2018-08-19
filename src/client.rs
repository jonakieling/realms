
use std::net::{TcpStream, Shutdown};
use std::sync::mpsc::Receiver;
use std::io::prelude::*;
use std::io;

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;
use termion::event;

use tui::layout::{Direction, Group, Size};
use tui::widgets::{Widget, Paragraph, Block, Borders, List, Item, SelectableList};
use tui::style::{Style, Color};

use Event;
use tokens::*;
use utility::SelectionStorage;

pub enum InteractiveUi {
	LOCATIONS,
	EXPLORERS
}

pub struct Periscope {
	pub stream: TcpStream,
	pub realm: Option<Realm>,
	pub id: usize,
	pub locations: SelectionStorage<Tile>,
	pub explorers: SelectionStorage<Explorer>,
	pub active_ui: InteractiveUi
}

impl Periscope {
	pub fn new(stream: TcpStream) -> Periscope {
		let mut periscope = Periscope {
			stream,
			realm: None,
			id: 0,
			locations: SelectionStorage::new(),
			explorers: SelectionStorage::new(),
			active_ui: InteractiveUi::LOCATIONS
		};

		periscope.send_request(RealmsProtocol::CONNECT(None));

		periscope
	}

	pub fn run(mut self, t: &mut Terminal<RawBackend>, rx: &Receiver<Event>) -> Result<(), io::Error> {
		loop {
			let t_size = t.size().unwrap();
			
			Group::default()
		        .direction(Direction::Vertical)
        		.sizes(&[Size::Fixed(8), Size::Min(0)])
		        .render(t, &t_size, |t, main_chunks| {

				Group::default()
			        .direction(Direction::Horizontal)
	        		.sizes(&[Size::Percent(75), Size::Percent(25)])
			        .render(t, &main_chunks[0], |t, chunks| {
			        	Paragraph::default()
					        .text(
					            "request new realm with {mod=bold r}\nchange island with {mod=bold i}\nchange expedition with {mod=bold e}\nexit with {mod=bold q}\nselect with {mod=bold ↓↑}, switch with {mod=bold ←→}\n",
					        ).block(Block::default().title("Abstract").borders(Borders::ALL))
					        .render(t, &chunks[0]);


			        	Paragraph::default()
					        .text(
					            &format!("id {{mod=bold {}}}", self.id),
					        ).block(Block::default().title("Client").borders(Borders::ALL))
					        .render(t, &chunks[1]);

			        });

			        if self.realm.is_some() {

			        	let location = self.locations.current().unwrap().clone();
			        	let location_index = self.locations.current_index();
			        	let locations: Vec<String> = self.locations.iter().map(|tile| {
		                    format!("{}", tile)
		                }).collect();

			        	let explorer_index = self.explorers.current_index();
			        	let explorers: Vec<String> = self.explorers.iter().map(|explorer| {
		                    format!("{}", explorer)
		                }).collect();

						Group::default()
					        .direction(Direction::Horizontal)
			        		.sizes(&[Size::Percent(30), Size::Percent(70)])
					        .render(t, &main_chunks[1], |t, chunks| {
					            let style = Style::default();

					        	let mut border_style = Style::default();
					        	if let InteractiveUi::LOCATIONS = self.active_ui {
					        	    border_style = Style::default().fg(Color::Yellow);
					        	}

	                            SelectableList::default()
	                                .block(Block::default().borders(Borders::ALL).title("Island").border_style(border_style))
	                                .items(&locations)
	                                .select(location_index)
	                                .highlight_style(
	                                    Style::default().fg(Color::Yellow),
	                                )
	                                .highlight_symbol(">")
	                                .render(t, &chunks[0]);

								Group::default()
							        .direction(Direction::Vertical)
					        		.sizes(&[Size::Percent(50), Size::Percent(50)])
							        .render(t, &chunks[1], |t, chunks| {

							        	let mut border_style = Style::default();
							        	if let InteractiveUi::EXPLORERS = self.active_ui {
							        	    border_style = Style::default().fg(Color::Yellow);
							        	}

			                            SelectableList::default()
			                                .block(Block::default().borders(Borders::ALL).title("Expedition").border_style(border_style))
			                                .items(&explorers)
			                                .select(explorer_index)
			                                .highlight_style(
			                                    Style::default().fg(Color::Yellow),
			                                ).highlight_symbol(">")
			                                .render(t, &chunks[0]);


								        	let particularities = location.particularities.iter().map(|particularity| {
								                Item::StyledData(
								                    format!("{:?}", particularity),
								                    &style
								                )
								            });
								    		List::new(particularities)
								                .block(Block::default().borders(Borders::ALL).title(&format!("{} {}", "Location", location)))
								                .render(t, &chunks[1]);
							        });
					        });
			        }
		        });
	        t.draw()?;

			let evt = rx.recv().unwrap();
			match evt {
			    Event::Tick => {
    				// todo: pulling updates (also keep-alive)
			    },
			    Event::Input(key) => {
			    	match key {
			    		event::Key::Up => {
		    				match self.active_ui {
		    				    InteractiveUi::LOCATIONS => {
		    				    	self.locations.prev();
		    				    },
		    				    InteractiveUi::EXPLORERS => {
		    				    	self.explorers.prev();
		    				    }
		    				}
			    		},
			    		event::Key::Down => {
		    				match self.active_ui {
		    				    InteractiveUi::LOCATIONS => {
		    				    	self.locations.next();
		    				    },
		    				    InteractiveUi::EXPLORERS => {
		    				    	self.explorers.next();
		    				    }
		    				}
			    		},
			    		event::Key::Right => {
		    				match self.active_ui {
		    				    InteractiveUi::LOCATIONS => self.active_ui = InteractiveUi::EXPLORERS,
		    				    InteractiveUi::EXPLORERS => self.active_ui = InteractiveUi::LOCATIONS
		    				}
			    		},
			    		event::Key::Left => {
		    				match self.active_ui {
		    				    InteractiveUi::LOCATIONS => self.active_ui = InteractiveUi::EXPLORERS,
		    				    InteractiveUi::EXPLORERS => self.active_ui = InteractiveUi::LOCATIONS
		    				}
			    		},
			    		event::Key::Char('r') => {
		    				self.send_request(RealmsProtocol::REALM(None));
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
	    		self.locations = SelectionStorage::new_from(&realm.island.tiles);
	    		self.explorers = SelectionStorage::new_from(&realm.expedition.explorers);
	    		self.realm = Some(realm);
	        },
	        RealmsProtocol::QUIT => {
				self.stream.shutdown(Shutdown::Both).expect("connection should have terminated.");
	        },
	        _ => {
	        }
	    }
	}
}