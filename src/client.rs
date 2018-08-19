
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
	Locations,
	Explorers,
	Realms
}

pub struct Periscope {
	pub stream: TcpStream,
	pub realm: Option<Realm>,
	pub realms: SelectionStorage<usize>,
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
			realms: SelectionStorage::new(),
			id: 0,
			locations: SelectionStorage::new(),
			explorers: SelectionStorage::new(),
			active_ui: InteractiveUi::Realms
		};

		periscope.send_request(RealmsProtocol::Register);
		periscope.send_request(RealmsProtocol::RequestRealmsList);

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
						            "move cursor with {mod=bold ↓↑}\nswitch with {mod=bold → ←}\npick with {mod=bold Enter}\nexit with {mod=bold q}",
						        ).block(Block::default().title("Abstract").borders(Borders::ALL))
						        .render(t, &chunks[0]);
			        		// end Paragraph::default()

				        	Paragraph::default()
						        .text(
						            &format!("id {{mod=bold {}}}", self.id),
						        ).block(Block::default().title("Client").borders(Borders::ALL))
						        .render(t, &chunks[1]);
			        		// end Paragraph::default()
			        	});
	        		// end Group::default()

			        match self.active_ui {
					    InteractiveUi::Locations | InteractiveUi::Explorers => {
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
							        .direction(Direction::Vertical)
					        		.sizes(&[Size::Fixed(2), Size::Min(0)])
							        .render(t, &main_chunks[1], |t, chunks| {

						        		if let Some(ref realm) = self.realm {
								        	Paragraph::default()
										        .text(
										            &format!("current realm {{mod=bold {}}}; switch to realms list with {{mod=bold l}}", realm.id)
										        ).block(Block::default())
										        .render(t, &chunks[0]);
							        		// end Paragraph::default()
						        		} else {
								        	Paragraph::default()
										        .text(
										            "switch to realms list with {mod=bold l}"
										        ).block(Block::default())
										        .render(t, &chunks[0]);
							        		// end Paragraph::default()
						        		}

						        		Group::default()
									        .direction(Direction::Horizontal)
							        		.sizes(&[Size::Percent(30), Size::Percent(70)])
									        .render(t, &chunks[1], |t, chunks| {
									            let style = Style::default();

									        	let mut border_style = Style::default();
									        	if let InteractiveUi::Locations = self.active_ui {
									        	    border_style = Style::default().fg(Color::Yellow);
									        	}

					                            SelectableList::default()
					                                .block(Block::default().borders(Borders::ALL).title("Island").border_style(border_style))
					                                .items(&locations)
					                                .select(location_index)
					                                .highlight_style(
					                                    Style::default().fg(Color::Yellow),
					                                )
					                                .highlight_symbol("→")
					                                .render(t, &chunks[0]);

												Group::default()
											        .direction(Direction::Vertical)
									        		.sizes(&[Size::Percent(50), Size::Percent(50)])
											        .render(t, &chunks[1], |t, chunks| {

											        	let mut border_style = Style::default();
											        	if let InteractiveUi::Explorers = self.active_ui {
											        	    border_style = Style::default().fg(Color::Yellow);
											        	}

							                            SelectableList::default()
							                                .block(Block::default().borders(Borders::ALL).title("Expedition").border_style(border_style))
							                                .items(&explorers)
							                                .select(explorer_index)
							                                .highlight_style(
							                                    Style::default().fg(Color::Yellow),
							                                ).highlight_symbol("→")
							                                .render(t, &chunks[0]);
											        	// end SelectableList::default()

											        	let particularities = location.particularities.iter().map(|particularity| {
											                Item::StyledData(
											                    format!("{:?}", particularity),
											                    &style
											                )
											            });
											    		List::new(particularities)
											                .block(Block::default().borders(Borders::ALL).title(&format!("{} {}", "Location", location)))
											                .render(t, &chunks[1]);
											        	// end List::new()
											        // end Group::default()
										        });
								        	});
						        		// end Group::default()

							        });
						        // end Group::default()
					        }
					    },
					    InteractiveUi::Realms => {
					    	Group::default()
						        .direction(Direction::Vertical)
				        		.sizes(&[Size::Fixed(2), Size::Min(0)])
						        .render(t, &main_chunks[1], |t, chunks| {
						        	Paragraph::default()
								        .text(
								            "request new realm with {mod=bold r}"
								        ).block(Block::default())
								        .render(t, &chunks[0]);
					        		// end Paragraph::default()

						        	let border_style = Style::default().fg(Color::Yellow);

						        	let realms_index = self.realms.current_index();
						        	let realms: Vec<String> = self.realms.iter().map(|realm| {
					                    format!("{}", realm)
					                }).collect();

		                            SelectableList::default()
		                                .block(Block::default().borders(Borders::ALL).title("Realms")
	                                	.border_style(border_style))
		                                .items(&realms)
		                                .select(realms_index)
		                                .highlight_style(
		                                    Style::default().fg(Color::Yellow),
		                                )
		                                .highlight_symbol("→")
		                                .render(t, &chunks[1]);
					        		// end SelectableList::default()
						        });
					        // end Group::default()
					    }
					}
		        });
    		// end Group::default()

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
		    				    InteractiveUi::Locations => {
		    				    	self.locations.prev();
		    				    },
		    				    InteractiveUi::Explorers => {
		    				    	self.explorers.prev();
		    				    },
		    				    InteractiveUi::Realms => {
		    				    	self.realms.prev();
		    				    }
		    				}
			    		},
			    		event::Key::Down => {
		    				match self.active_ui {
		    				    InteractiveUi::Locations => {
		    				    	self.locations.next();
		    				    },
		    				    InteractiveUi::Explorers => {
		    				    	self.explorers.next();
		    				    },
		    				    InteractiveUi::Realms => {
		    				    	self.realms.next();
		    				    }
		    				}
			    		},
			    		event::Key::Right => {
		    				match self.active_ui {
		    				    InteractiveUi::Explorers => self.active_ui = InteractiveUi::Locations,
		    				    InteractiveUi::Locations => self.active_ui = InteractiveUi::Explorers,
		    				    InteractiveUi::Realms => { }
		    				}
			    		},
			    		event::Key::Left => {
		    				match self.active_ui {
		    				    InteractiveUi::Explorers => self.active_ui = InteractiveUi::Locations,
		    				    InteractiveUi::Locations => self.active_ui = InteractiveUi::Explorers,
		    				    InteractiveUi::Realms => { }
		    				}
			    		},
			    		event::Key::Char('r') => {
			    			match self.active_ui {
		    				    InteractiveUi::Explorers => { },
		    				    InteractiveUi::Locations => { },
		    				    InteractiveUi::Realms => {
				    				self.send_request(RealmsProtocol::RequestNewRealm);
				    				self.send_request(RealmsProtocol::RequestRealmsList);
				    				self.realms.last();
		    				    }
		    				}
			    		},
			    		event::Key::Char('l') => {
			    			self.active_ui = InteractiveUi::Realms;
			    		},
			    		event::Key::Char('\n') => {
			    			match self.active_ui {
		    				    InteractiveUi::Explorers => { },
		    				    InteractiveUi::Locations => { },
		    				    InteractiveUi::Realms => {
			    					self.active_ui = InteractiveUi::Locations;
		    				    	let realm_id = *self.realms.current().unwrap();
		    				    	let mut loaded = false;
		    				    	if let Some(ref realm) = self.realm {
		    				    	    if realm.id == realm_id {
		    				    	    	loaded = true;
		    				    	    }
		    				    	}

		    				    	if !loaded {
		    							self.send_request(RealmsProtocol::RequestRealm(realm_id));	
		    				    	}
		    				    }
		    				}
			    		},
			    		event::Key::Char('q') => {
		    				self.send_request(RealmsProtocol::Quit);
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
		let data = serialize(&(self.id, request)).expect("could not serialize data package for request.");
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
	        RealmsProtocol::Connect(id) => {
	    		self.id = id;
	        },
	        RealmsProtocol::RealmsList(realms) => {
	        	self.realms = SelectionStorage::new_from(&realms);
	        },
	        RealmsProtocol::Realm(realm) => {
	    		self.locations = SelectionStorage::new_from(&realm.island.tiles);
	    		self.explorers = SelectionStorage::new_from(&realm.expedition.explorers);
	    		self.realm = Some(realm);
	        },
	        RealmsProtocol::Quit => {
				self.stream.shutdown(Shutdown::Both).expect("connection should have terminated.");
	        },
	        _ => {
	        }
	    }
	}
}