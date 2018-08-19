
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

pub struct Data {
	pub id: ClientId,
	pub realm: Option<Realm>,
	pub realms: SelectionStorage<RealmId>,
	pub location: Option<Tile>,
	pub locations: SelectionStorage<Tile>,
	pub explorers: SelectionStorage<Explorer>,
	pub active: InteractiveUi
}

pub struct Periscope {
	pub stream: TcpStream,
	pub data: Data
}

impl Periscope {
	pub fn new(stream: TcpStream) -> Periscope {
		let mut periscope = Periscope {
			stream,
			data: Data {
				id: 0,
				realm: None,
				realms: SelectionStorage::new(),
				location: None,
				locations: SelectionStorage::new(),
				explorers: SelectionStorage::new(),
				active: InteractiveUi::Realms
			}
		};

		send_request(&mut periscope.stream, &mut periscope.data, RealmsProtocol::Register);
		send_request(&mut periscope.stream, &mut periscope.data, RealmsProtocol::RequestRealmsList);

		periscope
	}

	pub fn run(mut self, t: &mut Terminal<RawBackend>, rx: &Receiver<Event>) -> Result<(), io::Error> {
		loop {
			
			draw_dashboard(t, &mut self.data)?;
	        t.draw()?;

			if !handle_events(rx, &mut self.stream, &mut self.data) {
				break;
			}
		}
	    t.show_cursor().unwrap();
	    t.clear().unwrap();

	    Ok(())
	}
}

fn send_request(stream: &mut TcpStream, data: &mut Data, request: RealmsProtocol) {
	let raw = serialize(&(data.id, request)).expect("could not serialize data package for request.");
	stream.write(&raw).expect("could not write to tcp stream.");
	stream.flush().unwrap();
	handle_response(stream, data);
}

fn handle_response(stream: &mut TcpStream, data: &mut Data) {
	let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();
    stream.flush().unwrap();


    let response: RealmsProtocol = deserialize(&buffer).expect("could not deserialize server response");

    match response {
        RealmsProtocol::Connect(id) => {
    		data.id = id;
        },
        RealmsProtocol::RealmsList(realms) => {
        	data.realms = SelectionStorage::new_from(&realms);
        },
        RealmsProtocol::Realm(realm) => {
    		data.locations = SelectionStorage::new_from(&realm.island.tiles);
    		data.explorers = SelectionStorage::new_from(&realm.expedition.explorers);
    		data.realm = Some(realm);
        },
        RealmsProtocol::Quit => {
			stream.shutdown(Shutdown::Both).expect("connection should have terminated.");
        },
        _ => { }
    }
}

fn handle_events(rx: &Receiver<Event>, stream: &mut TcpStream, data: &mut Data) -> bool {
	let mut should_continue = true;

	let evt = rx.recv().unwrap();
	match evt {
	    Event::Tick => {
			// todo: pulling updates (also keep-alive)
	    },
	    Event::Input(key) => {
	    	match key {
	    		event::Key::Up => {
    				match data.active {
    				    InteractiveUi::Locations => {
    				    	data.locations.prev();
    				    },
    				    InteractiveUi::Explorers => {
    				    	data.explorers.prev();
    				    },
    				    InteractiveUi::Realms => {
    				    	data.realms.prev();
    				    }
    				}
	    		},
	    		event::Key::Down => {
    				match data.active {
    				    InteractiveUi::Locations => {
    				    	data.locations.next();
    				    },
    				    InteractiveUi::Explorers => {
    				    	data.explorers.next();
    				    },
    				    InteractiveUi::Realms => {
    				    	data.realms.next();
    				    }
    				}
	    		},
	    		event::Key::Right => {
    				match data.active {
    				    InteractiveUi::Explorers => data.active = InteractiveUi::Locations,
    				    InteractiveUi::Locations => data.active = InteractiveUi::Explorers,
    				    InteractiveUi::Realms => { }
    				}
	    		},
	    		event::Key::Left => {
    				match data.active {
    				    InteractiveUi::Explorers => data.active = InteractiveUi::Locations,
    				    InteractiveUi::Locations => data.active = InteractiveUi::Explorers,
    				    InteractiveUi::Realms => { }
    				}
	    		},
	    		event::Key::Char('r') => {
	    			match data.active {
    				    InteractiveUi::Explorers => { },
    				    InteractiveUi::Locations => { },
    				    InteractiveUi::Realms => {
		    				send_request(stream, data, RealmsProtocol::RequestNewRealm);
		    				send_request(stream, data, RealmsProtocol::RequestRealmsList);
		    				data.realms.last();
    				    }
    				}
	    		},
	    		event::Key::Char('l') => {
	    			data.active = InteractiveUi::Realms;
	    		},
	    		event::Key::Char('\n') => {
	    			match data.active {
    				    InteractiveUi::Explorers => {
    				    	let location_id = data.locations.current().unwrap().id;
    				    	let realm_id = data.realm.as_ref().unwrap().id;
    				    	let action = data.explorers.current().unwrap().action();
    				    	let last_location_index = data.locations.current_index();
    				    	let last_explorers_index = data.explorers.current_index();
							send_request(stream, data, RealmsProtocol::Move(Move::Action(realm_id, location_id, action)));
							data.locations.at(last_location_index);
							data.explorers.at(last_explorers_index);
    				    },
    				    InteractiveUi::Locations => {
    				    	let location_id = data.locations.current().unwrap().id;
    				    	data.location = Some(data.locations.current().unwrap().clone());
    				    	let realm_id = data.realm.as_ref().unwrap().id;
    				    	// request reset locations index, we set it back afterwards
    				    	let last_index = data.locations.current_index();
							send_request(stream, data, RealmsProtocol::Move(Move::ChangeLocation(realm_id, location_id)));
							data.locations.at(last_index);
    				    },
    				    InteractiveUi::Realms => {
	    					data.active = InteractiveUi::Locations;
    				    	let realm_id = *data.realms.current().unwrap();
    				    	let mut loaded = false;
    				    	if let Some(ref realm) = data.realm {
    				    	    if realm.id == realm_id {
    				    	    	loaded = true;
    				    	    }
    				    	}
				    		// only fetch realm when not already data.active
    				    	if !loaded {
    							send_request(stream, data, RealmsProtocol::RequestRealm(realm_id));	
    				    	}
    				    }
    				}
	    		},
	    		event::Key::Char('q') => {
    				send_request(stream, data, RealmsProtocol::Quit);
    				should_continue = false;
	    		},
	    		_ => { }
	    	}
		}
	}

	should_continue
}

fn draw_dashboard(terminal: &mut Terminal<RawBackend>, data: &mut Data) -> Result<(), io::Error> {
	let terminal_area = terminal.size().unwrap();
			
	Group::default()
        .direction(Direction::Vertical)
		.sizes(&[Size::Fixed(6), Size::Min(0)])
        .render(terminal, &terminal_area, |t, chunks| {

			Group::default()
		        .direction(Direction::Horizontal)
        		.sizes(&[Size::Percent(75), Size::Percent(25)])
		        .render(t, &chunks[0], |t, chunks| {
		        	Paragraph::default()
				        .text(
				            "move cursor with {mod=bold ↓↑}\nswitch with {mod=bold → ←}\npick with {mod=bold Enter}\nexit with {mod=bold q}",
				        ).block(Block::default().title("Abstract").borders(Borders::ALL))
				        .render(t, &chunks[0]);
	        		// end Paragraph::default()

		        	Paragraph::default()
				        .text(
				            &format!("id {{mod=bold {}}}", data.id),
				        ).block(Block::default().title("Client").borders(Borders::ALL))
				        .render(t, &chunks[1]);
	        		// end Paragraph::default()
	        	});
    		// end Group::default()

	        match data.active {
			    InteractiveUi::Locations | InteractiveUi::Explorers => {
			    	if data.realm.is_some() {

			        	let location = data.locations.current().unwrap();
			        	let location_index = data.locations.current_index();
			        	let locations: Vec<String> = data.locations.iter().map(|tile| {
			        		if let Some(ref location) = data.location {
			        			if location.id == tile.id {
				        			format!("{} *", tile)
			        			} else {
				        			format!("{}", tile)
			        			}
			        		} else {
			        			format!("{}", tile)
			        		}
		                }).collect();

			        	let explorer_index = data.explorers.current_index();
			        	let explorers: Vec<String> = data.explorers.iter().map(|explorer| {
		                    format!("{}", explorer)
		                }).collect();

		        		Group::default()
					        .direction(Direction::Vertical)
			        		.sizes(&[Size::Fixed(2), Size::Min(0)])
					        .render(t, &chunks[1], |t, chunks| {

				        		if let Some(ref realm) = data.realm {
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
							            let highlight = Style::default().fg(Color::Yellow);

							        	let mut border_style = Style::default();
							        	if let InteractiveUi::Locations = data.active {
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
									        	if let InteractiveUi::Explorers = data.active {
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


												Group::default()
											        .direction(Direction::Horizontal)
									        		.sizes(&[Size::Percent(50), Size::Percent(50)])
											        .render(t, &chunks[1], |t, chunks| {

									        			let mut info = vec![];
									        			info.push(Item::StyledData(
											                    format!("Buildings {:?}", location.buildings),
											                    &style
										                ));
									        			info.push(Item::StyledData(
											                    format!("Resources {}", location.resources),
											                    &style
										                ));
										                if location.mapped {
										        			info.push(Item::StyledData(
												                    format!("Mapped"),
												                    &highlight
											                ));
										                }

											    		List::new(info.into_iter())
											                .block(Block::default().borders(Borders::ALL).title(&format!("{} {}", "Location", location)))
											                .render(t, &chunks[0]);
										        		// end List::new()

											        	let particularities = location.particularities.iter().map(|particularity| {
											                Item::StyledData(
											                    format!("{:?}", particularity),
											                    &style
											                )
											            });
											    		List::new(particularities)
											                .block(Block::default().borders(Borders::ALL).title(&format!("Particularities")))
											                .render(t, &chunks[1]);
										        		// end List::new()

											        });
									        	// end Group::default()
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
				        .render(t, &chunks[1], |t, chunks| {
				        	Paragraph::default()
						        .text(
						            "request new realm with {mod=bold r}"
						        ).block(Block::default())
						        .render(t, &chunks[0]);
			        		// end Paragraph::default()

				        	let border_style = Style::default().fg(Color::Yellow);

				        	let realms_index = data.realms.current_index();
				        	let realms: Vec<String> = data.realms.iter().map(|realm| {
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

	Ok(())
}