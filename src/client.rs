
use std::net::{TcpStream, Shutdown};
use std::sync::mpsc::Receiver;
use std::io::prelude::*;
use std::io;

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;
use termion::event;

use tui::layout::{Direction, Group, Size, Rect};
use tui::widgets::{Widget, Paragraph, Block, Borders, List, Item, SelectableList};
use tui::style::{Style, Color};

use Event;
use tokens::*;
use utility::SelectionStorage;

pub enum InteractiveUi {
	Locations,
	Explorers,
	Realms,
	MoveLocations
}

pub struct Data {
	pub id: ClientId,
	pub realm: Option<Realm>,
	pub realms: SelectionStorage<RealmId>,
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

// todo extract functions for modules
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

				        	sync_locations_with_explorer(data);
    				    },
    				    InteractiveUi::Realms => {
    				    	data.realms.prev();
    				    },
    				    InteractiveUi::MoveLocations => {
    				    	data.locations.prev();
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

				        	sync_locations_with_explorer(data);
    				    },
    				    InteractiveUi::Realms => {
    				    	data.realms.next();
    				    },
    				    InteractiveUi::MoveLocations => {
    				    	data.locations.next();
    				    }
    				}
	    		},
	    		event::Key::Right => {
    				match data.active {
    				    InteractiveUi::Explorers => {
    				    	data.active = InteractiveUi::MoveLocations;

				        	sync_locations_with_explorer(data);
    				    },
    				    InteractiveUi::Locations => {
    				    	data.active = InteractiveUi::Explorers;

				        	sync_locations_with_explorer(data);
    				    },
    				    InteractiveUi::MoveLocations => {
    				    	data.active = InteractiveUi::Locations;
    				    	data.locations.at(0);
    				    },
    				    InteractiveUi::Realms => { }
    				}
	    		},
	    		event::Key::Left => {
    				match data.active {
    				    InteractiveUi::Explorers => {
    				    	data.active = InteractiveUi::Locations;
    				    	data.locations.at(0);
    				    },
    				    InteractiveUi::MoveLocations => {
    				    	data.active = InteractiveUi::Explorers;

				        	sync_locations_with_explorer(data);
    				    },
    				    InteractiveUi::Locations => {
    				    	data.active = InteractiveUi::MoveLocations;

				        	sync_locations_with_explorer(data);
    				    },
    				    InteractiveUi::Realms => { }
    				}
	    		},
	    		event::Key::Char('r') => {
	    			match data.active {
    				    InteractiveUi::Explorers => { },
    				    InteractiveUi::Locations => { },
    				    InteractiveUi::MoveLocations => { },
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
    				    	// todo extract function for cleaner borrowing
    				    	let location_id = data.locations.current().unwrap().id;
    				    	let realm_id = data.realm.as_ref().unwrap().id;
    				    	let action = data.explorers.current().unwrap().action();
    				    	// request reset locations and explorers index, we set it back afterwards
    				    	let last_location_index = data.locations.current_index();
    				    	let last_explorers_index = data.explorers.current_index();
							send_request(stream, data, RealmsProtocol::Move(Move::Action(realm_id, location_id, action)));
							data.locations.at(last_location_index);
							data.explorers.at(last_explorers_index);
    				    },
    				    InteractiveUi::Locations => { },
    				    InteractiveUi::Realms => {
    				    	// todo extract function for cleaner borrowing
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
    				    },
    				    InteractiveUi::MoveLocations => {
    				    	// todo extract function for cleaner borrowing
    				    	let location_id = data.locations.current().unwrap().id;
    				    	let realm_id = data.realm.as_ref().unwrap().id;
    				    	let explorer_id = data.explorers.current().unwrap().id;
    				    	// request reset locations index, we set it back afterwards
    				    	let last_index = data.locations.current_index();
    				    	let last_explorers_index = data.explorers.current_index();
							send_request(stream, data, RealmsProtocol::Move(Move::ChangeLocation(realm_id, location_id, explorer_id)));
							data.locations.at(last_index);
							data.explorers.at(last_explorers_index);
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

fn sync_locations_with_explorer(data: &mut Data) {
	let mut current_explorer_location = 0;
	if let Some(explorer_location) = data.explorers.current().unwrap().location {
		current_explorer_location = explorer_location;
	}
	let mut current_explorer_location_index = 0;
	for (index, location) in data.locations.iter().enumerate() {
	    if current_explorer_location == location.id {
	        current_explorer_location_index = index;
	    }
	}
	data.locations.at(current_explorer_location_index);
}

fn draw_dashboard(terminal: &mut Terminal<RawBackend>, data: &mut Data) -> Result<(), io::Error> {
	let terminal_area = terminal.size().unwrap();
			
	Group::default()
        .direction(Direction::Vertical)
		.sizes(&[Size::Fixed(6), Size::Min(0)])
        .render(terminal, &terminal_area, |t, chunks| {

        	draw_header(t, &chunks[0], &data);

	        match data.active {
			    InteractiveUi::Locations | InteractiveUi::Explorers | InteractiveUi::MoveLocations => {
			    	if data.realm.is_some() {
			    		draw_realm(t, &chunks[1], &data);
			        }
			    },
			    InteractiveUi::Realms => {
    				draw_realms_list(t, &chunks[1], &data);
			    }
			}
        });
	// end Group::default()

	Ok(())
}

fn draw_header(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	Group::default()
        .direction(Direction::Horizontal)
		.sizes(&[Size::Percent(75), Size::Percent(25)])
        .render(t, area, |t, chunks| {
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
}

fn draw_realms_list(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	Group::default()
        .direction(Direction::Vertical)
		.sizes(&[Size::Fixed(2), Size::Min(0)])
        .render(t, area, |t, chunks| {
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

fn draw_realm(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {

	Group::default()
        .direction(Direction::Vertical)
		.sizes(&[Size::Fixed(2), Size::Min(0)])
        .render(t, area, |t, chunks| {

        	draw_realm_info(t, &chunks[0], &data);

        	draw_realm_ui(t, &chunks[1], &data);

        });
    // end Group::default()
}

fn draw_realm_info(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	if let Some(ref realm) = data.realm {
    	Paragraph::default()
	        .text(
	            &format!("current realm {{mod=bold {}}}; switch to realms list with {{mod=bold l}}", realm.id)
	        ).block(Block::default())
	        .render(t, area);
		// end Paragraph::default()
	} else {
    	Paragraph::default()
	        .text(
	            "switch to realms list with {mod=bold l}"
	        ).block(Block::default())
	        .render(t, area);
		// end Paragraph::default()
	}
}


fn draw_realm_ui(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	Group::default()
	    .direction(Direction::Horizontal)
		.sizes(&[Size::Percent(30), Size::Percent(70)])
	    .render(t, area, |t, chunks| {

			let location_index = data.locations.current_index();
			let locations: Vec<String> = data.locations.iter().map(|tile| {
				format!("{}", tile)
		    }).collect();

	    	let mut border_style = Style::default();
	    	if let InteractiveUi::Locations = data.active {
	    	    border_style = Style::default().fg(Color::Yellow);
	    	}
	    	let mut locations_list_style = Style::default();
	    	if let InteractiveUi::Locations = data.active {
	    	    locations_list_style = Style::default().fg(Color::Yellow);
	    	}
	        SelectableList::default()
	            .block(Block::default().borders(Borders::ALL).title("Island").border_style(border_style))
	            .items(&locations)
	            .select(location_index)
	            .highlight_style(
	                locations_list_style
	            )
	            .render(t, &chunks[0]);
    		// end SelectableList::default()

			Group::default()
		        .direction(Direction::Vertical)
	    		.sizes(&[Size::Percent(50), Size::Percent(50)])
		        .render(t, &chunks[1], |t, chunks| {
					draw_realm_expedition(t, &chunks[0], &data);
					if let InteractiveUi::Explorers = data.active {
		        		if data.explorers.current().unwrap().location.is_some() {
							draw_realm_location(t, &chunks[1], &data);
		        		} else {
					    	Paragraph::default()
						        .text(
						            "this explorer has not embarked yet. select a location from the list to move them there."
						        ).block(Block::default().borders(Borders::ALL).title("Location").border_style(Style::default()))
								.wrap(true)
						        .render(t, &chunks[1]);
							// end Paragraph::default()	
			        	}
		        	} else {
						draw_realm_location(t, &chunks[1], &data);
		        	}
	        	});
	        // end Group::default()
		});
	// end Group::default()
}

fn draw_realm_expedition(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {

	let explorer_index = data.explorers.current_index();
	let explorers: Vec<String> = data.explorers.iter().map(|explorer| {
		if let Some(explorer_location) = explorer.location {
        	format!("{} {}", explorer.variant, explorer_location)
		} else {
        	format!("{}", explorer.variant)
		}
    }).collect();

	let location_index = data.locations.current_index();
	let locations: Vec<String> = data.locations.iter().map(|tile| {
		format!("{}", tile)
    }).collect();

	Group::default()
        .direction(Direction::Horizontal)
		.sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, area, |t, chunks| {
        	if let InteractiveUi::Explorers = data.active {
                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title("Expedition").border_style(Style::default().fg(Color::Yellow)))
                    .items(&explorers)
                    .select(explorer_index)
                    .highlight_style(
                        Style::default().fg(Color::Yellow),
                    ).highlight_symbol("→")
                    .render(t, &chunks[0]);
	        	// end SelectableList::default()
        	} else if let InteractiveUi::MoveLocations = data.active {
                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title("Expedition").border_style(Style::default().fg(Color::Yellow)))
                    .items(&explorers)
                    .select(explorer_index)
                    .highlight_style(
                        Style::default().fg(Color::Yellow),
                    )
                    .render(t, &chunks[0]);
	        	// end SelectableList::default()
        	} else {
                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title("Expedition").border_style(Style::default()))
                    .items(&explorers)
                    .select(explorer_index)
                    .highlight_style(
                        Style::default()
                    )
                    .render(t, &chunks[0]);
	        	// end SelectableList::default()
        	}

        	if let InteractiveUi::MoveLocations = data.active {
                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title("Move Explorer").border_style(Style::default().fg(Color::Yellow)))
                    .items(&locations)
                    .select(location_index)
                    .highlight_style(
                        Style::default().fg(Color::Yellow)
                    )
                    .highlight_symbol("→")
                    .render(t, &chunks[1]);
	        	// end SelectableList::default()
        	} else if let InteractiveUi::Explorers = data.active {
        		if data.explorers.current().unwrap().location.is_some() {
                    SelectableList::default()
                        .block(Block::default().borders(Borders::ALL).title("Move Explorer").border_style(Style::default()))
                        .items(&locations)
                        .select(location_index)
                        .highlight_style(
                            Style::default().fg(Color::Yellow)
                        )
                        .render(t, &chunks[1]);
		        	// end SelectableList::default()
        		} else {
                    SelectableList::default()
                        .block(Block::default().borders(Borders::ALL).title("Move Explorer").border_style(Style::default()))
                        .items(&locations)
                        .select(location_index)
                        .highlight_style(
                            Style::default()
                        )
                        .render(t, &chunks[1]);
		        	// end SelectableList::default()
        		}
        	} else {
		    	Paragraph::default()
			        .text(
			            "select an explorer from the expedition to move them and make actions."
			        ).block(Block::default().borders(Borders::ALL).title("Move Explorer").border_style(Style::default()))
					.wrap(true)
			        .render(t, &chunks[1]);
				// end Paragraph::default()	
        	}

        });
	// end Group::default()
}

fn draw_realm_location(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {

	let location = data.locations.current().unwrap();

	Group::default()
        .direction(Direction::Horizontal)
		.sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, area, |t, chunks| {

	        let style = Style::default();
        	let cyan = Style::default().fg(Color::Cyan);
        	let green = Style::default().fg(Color::Green);

			let mut info = vec![];
			info.push(Item::StyledData(
                    format!("{:?}", location.buildings),
                    &style
            ));
			info.push(Item::StyledData(
                    format!("Resources {}", location.resources),
                    &style
            ));
            if location.mapped {
    			info.push(Item::StyledData(
	                    format!("Mapped"),
	                    &green
                ));
            }
            for explorer in data.explorers.iter() {
            	if let Some(explorer_location) = explorer.location {
            		if explorer_location == location.id {
	        			info.push(Item::StyledData(
			                    format!("{}", explorer.variant),
			                    &cyan
		                ));
            		}
            	}
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
}