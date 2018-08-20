
use std::net::{TcpStream, Shutdown};
use std::sync::mpsc::Receiver;
use std::io::prelude::*;
use std::io;

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;
use termion::event;

use Event;
use tokens::*;
use utility::SelectionStorage;

use client_dashboard::draw;

#[derive(Debug)]
pub enum InteractiveUi {
	Locations,
	Explorers,
	Realms,
	MoveLocations
}

#[derive(Debug)]
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

		if let RealmsProtocol::Connect(id) = send_request(&mut periscope.stream, 0, RealmsProtocol::Register) {
			periscope.data.id = id;
		}
		if let RealmsProtocol::RealmsList(response_realms) = send_request(&mut periscope.stream, periscope.data.id, RealmsProtocol::RequestRealmsList) {
			periscope.data.realms = SelectionStorage::new_from(&response_realms);
		}

		periscope
	}

	pub fn run(mut self, t: &mut Terminal<RawBackend>, rx: &Receiver<Event>) -> Result<(), io::Error> {
		loop {

			if !handle_events(rx, &mut self.stream, &mut self.data) {
				break;
			}

			draw(t, &mut self.data)?;
		}

	    Ok(())
	}
}

fn send_request(stream: &mut TcpStream, client: ClientId, request: RealmsProtocol) -> RealmsProtocol {
	let raw = serialize(&(client, request)).expect("could not serialize data package for request.");
	stream.write(&raw).expect("could not write request to tcp stream.");
	stream.flush().expect("could not flush request stream.");
	handle_response(stream)
}

fn handle_response(stream: &mut TcpStream) -> RealmsProtocol {
	let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("could not read response into buffer.");
    stream.flush().expect("could not flush response stream.");
    let response: RealmsProtocol = deserialize(&buffer).expect("could not deserialize server response");
    response
}

fn handle_events(rx: &Receiver<Event>, stream: &mut TcpStream, data: &mut Data) -> bool {
	let mut should_continue = true;

	let evt = rx.recv().expect("could not receive input event.");
	match evt {
	    Event::Tick => {
			// todo: pulling updates (also keep-alive)
	    },
	    Event::Input(key) => {
	    	match data.active {
			    InteractiveUi::Locations => {
			    	handle_locations_events(stream, data, key);
			    },
			    InteractiveUi::Explorers => {
			    	handle_explorer_events(stream, data, key);
			    },
			    InteractiveUi::Realms => {
			    	handle_realms_events(stream, data, key);
			    },
			    InteractiveUi::MoveLocations => {
			    	handle_move_locations_events(stream, data, key);
			    }
			}

	    	if let event::Key::Char('q') = key {
				if let RealmsProtocol::Quit = send_request(stream, data.id, RealmsProtocol::Quit) {
					stream.shutdown(Shutdown::Both).expect("connection should have terminated.");
					should_continue = false;
				}
	    	}
		}
	}

	should_continue
}

fn handle_realms_events(stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.realms.prev();
		},
		event::Key::Down => {
	    	data.realms.next();
		},
		event::Key::Char('r') => {
			if let RealmsProtocol::Realm(response_realm) = send_request(stream, data.id, RealmsProtocol::RequestNewRealm) {
				data.locations =  SelectionStorage::new_from(&response_realm.island.tiles);
	    		data.explorers = SelectionStorage::new_from(&response_realm.expedition.explorers);
				data.realm = Some(response_realm);
			}
			if let RealmsProtocol::RealmsList(response_realms) = send_request(stream, data.id, RealmsProtocol::RequestRealmsList) {
				data.realms = SelectionStorage::new_from(&response_realms);
			}
			data.realms.last();
		},
		event::Key::Char('\n') => {
	    	if data.realm.is_some() {
	    		let realm_id = data.realms.current().expect("could not access current realm selection.");
	    		let current_realm_id = data.realm.as_ref().expect("could not access active realm.").id;
	    		if *realm_id != current_realm_id {
		    		if let RealmsProtocol::Realm(response_realm) = send_request(stream, data.id, RealmsProtocol::RequestRealm(*realm_id)) {
						data.locations =  SelectionStorage::new_from(&response_realm.island.tiles);
			    		data.explorers = SelectionStorage::new_from(&response_realm.expedition.explorers);
						data.realm = Some(response_realm);
					}
	    		}
	    	} else {
	    		let realm_id = data.realms.current().expect("could not access current realm selection.");
	    		if let RealmsProtocol::Realm(response_realm) = send_request(stream, data.id, RealmsProtocol::RequestRealm(*realm_id)) {
		    		data.locations =  SelectionStorage::new_from(&response_realm.island.tiles);
		    		data.explorers = SelectionStorage::new_from(&response_realm.expedition.explorers);
					data.realm = Some(response_realm);
				}
	    	}
	    	data.active = InteractiveUi::Locations;
		},
		_ => { }
	}
}

fn handle_locations_events(_stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.locations.prev();
		},
		event::Key::Down => {
	    	data.locations.next();
		},
		event::Key::Right => {
	    	data.active = InteractiveUi::Explorers;
        	sync_locations_with_explorer(data);
		},
		event::Key::Left => {
	    	data.active = InteractiveUi::MoveLocations;
        	sync_locations_with_explorer(data);
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		_ => { }
	}
}

fn handle_move_locations_events(stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.locations.prev();
		},
		event::Key::Down => {
	    	data.locations.next();
		},
		event::Key::Right => {
	    	data.active = InteractiveUi::Locations;
	    	data.locations.at(0);
		},
		event::Key::Left => {
	    	data.active = InteractiveUi::Explorers;
        	sync_locations_with_explorer(data);
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		event::Key::Char('\n') => {
    		{
				// request reset locations index, we set it back afterwards
				let last_index = data.locations.current_index();
				let last_explorers_index = data.explorers.current_index();

				let realm_id = data.realms.current().expect("could not access current realm selection.");
	    		if let RealmsProtocol::Realm(response_realm) = explorer_move(stream, data.id, *realm_id, &mut data.locations, &mut data.explorers) {
		    		data.locations =  SelectionStorage::new_from(&response_realm.island.tiles);
		    		data.explorers = SelectionStorage::new_from(&response_realm.expedition.explorers);
					data.realm = Some(response_realm);
				}

				data.locations.at(last_index);
				data.explorers.at(last_explorers_index);
    		}
        	sync_locations_with_explorer(data);
	    	data.active = InteractiveUi::Explorers;
		},
		_ => { }
	}
}

fn handle_explorer_events(stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.explorers.prev();
        	sync_locations_with_explorer(data);
		},
		event::Key::Down => {
	    	data.explorers.next();
        	sync_locations_with_explorer(data);
		},
		event::Key::Right => {
	    	data.active = InteractiveUi::MoveLocations;
        	sync_locations_with_explorer(data);
		},
		event::Key::Left => {
	    	data.active = InteractiveUi::Locations;
	    	data.locations.at(0);
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		event::Key::Char('\n') => {
    		{
				// request reset locations and explorers index, we set it back afterwards
				let last_location_index = data.locations.current_index();
				let last_explorers_index = data.explorers.current_index();

    			let realm_id = data.realms.current().expect("could not access current realm selection.");
	    		if let RealmsProtocol::Realm(response_realm) = explorer_action(stream, data.id, *realm_id, &mut data.locations, &mut data.explorers) {
		    		data.locations =  SelectionStorage::new_from(&response_realm.island.tiles);
		    		data.explorers = SelectionStorage::new_from(&response_realm.expedition.explorers);
					data.realm = Some(response_realm);
				}

				data.locations.at(last_location_index);
				data.explorers.at(last_explorers_index);
    		}
        	sync_locations_with_explorer(data);
		},
		_ => { }
	}
}

fn sync_locations_with_explorer(data: &mut Data) {
	let mut current_explorer_location = 0;
	if let Some(explorer_location) = data.explorers.current().expect("could not access current explorers selection.").location {
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

fn explorer_action(stream: &mut TcpStream, client: ClientId, realm_id: RealmId, locations: &mut SelectionStorage<Tile>, explorers: &mut SelectionStorage<Explorer>) -> RealmsProtocol {
	let mut request = RealmsProtocol::Void;

	if let Some(location) = locations.current() {
		if let Some(explorer) = explorers.current() {
			request = send_request(stream, client, RealmsProtocol::Move(Move::Action(realm_id, location.id, explorer.action())));
		}
	}

	request
}

fn explorer_move(stream: &mut TcpStream, client: ClientId, realm_id: RealmId, locations: &mut SelectionStorage<Tile>, explorers: &mut SelectionStorage<Explorer>) -> RealmsProtocol {
	let mut request = RealmsProtocol::Void;

	if let Some(location) = locations.current() {
		if let Some(explorer) = explorers.current() {
			request = send_request(stream, client, RealmsProtocol::Move(Move::ChangeLocation(realm_id, location.id, explorer.id)));
		}
	}

	request
}