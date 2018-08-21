
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
	Regions,
	Explorers,
	Realms,
	MoveRegions
}

#[derive(Debug)]
pub struct Data {
	pub id: ClientId,
	pub realm: Option<Realm>,
	pub realms: SelectionStorage<RealmId>,
	pub regions: SelectionStorage<Region>,
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
				regions: SelectionStorage::new(),
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

	pub fn run(mut self, terminal: &mut Terminal<RawBackend>, rx: &Receiver<Event>) -> Result<(), io::Error> {
		loop {

			if !handle_events(rx, &mut self.stream, &mut self.data) {
				break;
			}

			draw(terminal, &mut self.data)?;
		}

	    terminal.show_cursor().unwrap();
	    terminal.clear().unwrap();

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
			    InteractiveUi::Regions => {
			    	handle_regions_events(stream, data, key);
			    },
			    InteractiveUi::Explorers => {
			    	handle_explorer_events(stream, data, key);
			    },
			    InteractiveUi::Realms => {
			    	handle_realms_events(stream, data, key);
			    },
			    InteractiveUi::MoveRegions => {
			    	handle_move_regions_events(stream, data, key);
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
				data.regions =  SelectionStorage::new_from(&response_realm.island.regions);
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
						data.regions =  SelectionStorage::new_from(&response_realm.island.regions);
			    		data.explorers = SelectionStorage::new_from(&response_realm.expedition.explorers);
						data.realm = Some(response_realm);
					}
	    		}
	    	} else {
	    		let realm_id = data.realms.current().expect("could not access current realm selection.");
	    		if let RealmsProtocol::Realm(response_realm) = send_request(stream, data.id, RealmsProtocol::RequestRealm(*realm_id)) {
		    		data.regions =  SelectionStorage::new_from(&response_realm.island.regions);
		    		data.explorers = SelectionStorage::new_from(&response_realm.expedition.explorers);
					data.realm = Some(response_realm);
				}
	    	}
	    	data.active = InteractiveUi::Regions;
		},
		_ => { }
	}
}

fn handle_regions_events(_stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.regions.prev();
		},
		event::Key::Down => {
	    	data.regions.next();
		},
		event::Key::Right => {
	    	data.active = InteractiveUi::Explorers;
        	sync_regions_with_explorer(data);
		},
		event::Key::Left => {
	    	data.active = InteractiveUi::MoveRegions;
        	sync_regions_with_explorer(data);
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		_ => { }
	}
}

fn handle_move_regions_events(stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.regions.prev();
		},
		event::Key::Down => {
	    	data.regions.next();
		},
		event::Key::Right => {
	    	data.active = InteractiveUi::Regions;
	    	data.regions.at(0);
		},
		event::Key::Left => {
	    	data.active = InteractiveUi::Explorers;
        	sync_regions_with_explorer(data);
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		event::Key::Char('\n') => {
    		{
				// request reset regions index, we set it back afterwards
				let last_index = data.regions.current_index();
				let last_explorers_index = data.explorers.current_index();

				let realm_id = data.realms.current().expect("could not access current realm selection.");
	    		if let RealmsProtocol::Realm(response_realm) = explorer_move(stream, data.id, *realm_id, &mut data.regions, &mut data.explorers) {
		    		data.regions =  SelectionStorage::new_from(&response_realm.island.regions);
		    		data.explorers = SelectionStorage::new_from(&response_realm.expedition.explorers);
					data.realm = Some(response_realm);
				}

				data.regions.at(last_index);
				data.explorers.at(last_explorers_index);
    		}
        	sync_regions_with_explorer(data);
	    	data.active = InteractiveUi::Explorers;
		},
		_ => { }
	}
}

fn handle_explorer_events(stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.explorers.prev();
        	sync_regions_with_explorer(data);
		},
		event::Key::Down => {
	    	data.explorers.next();
        	sync_regions_with_explorer(data);
		},
		event::Key::Right => {
	    	data.active = InteractiveUi::MoveRegions;
        	sync_regions_with_explorer(data);
		},
		event::Key::Left => {
	    	data.active = InteractiveUi::Regions;
	    	data.regions.at(0);
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		event::Key::Char('\n') => {
    		{
				// request reset regions and explorers index, we set it back afterwards
				let last_region_index = data.regions.current_index();
				let last_explorers_index = data.explorers.current_index();

    			let realm_id = data.realms.current().expect("could not access current realm selection.");
	    		if let RealmsProtocol::Realm(response_realm) = explorer_action(stream, data.id, *realm_id, &mut data.regions, &mut data.explorers) {
		    		data.regions =  SelectionStorage::new_from(&response_realm.island.regions);
		    		data.explorers = SelectionStorage::new_from(&response_realm.expedition.explorers);
					data.realm = Some(response_realm);
				}

				data.regions.at(last_region_index);
				data.explorers.at(last_explorers_index);
    		}
        	sync_regions_with_explorer(data);
		},
		_ => { }
	}
}

fn sync_regions_with_explorer(data: &mut Data) {
	let mut current_explorer_region = 0;
	if let Some(explorer_region) = data.explorers.current().expect("could not access current explorers selection.").region {
		current_explorer_region = explorer_region;
	}
	let mut current_explorer_region_index = 0;
	for (index, region) in data.regions.iter().enumerate() {
	    if current_explorer_region == region.id {
	        current_explorer_region_index = index;
	    }
	}
	data.regions.at(current_explorer_region_index);
}

fn explorer_action(stream: &mut TcpStream, client: ClientId, realm_id: RealmId, regions: &mut SelectionStorage<Region>, explorers: &mut SelectionStorage<Explorer>) -> RealmsProtocol {
	let mut request = RealmsProtocol::Void;

	if let Some(region) = regions.current() {
		if let Some(explorer) = explorers.current() {
			if let Some(action) = explorer.actions().first() {
				request = send_request(stream, client, RealmsProtocol::Explorer(Move::Action(realm_id, region.id, explorer.id, action.clone())));
			}
		}
	}

	request
}

fn explorer_move(stream: &mut TcpStream, client: ClientId, realm_id: RealmId, regions: &mut SelectionStorage<Region>, explorers: &mut SelectionStorage<Explorer>) -> RealmsProtocol {
	let mut request = RealmsProtocol::Void;

	if let Some(region) = regions.current() {
		if let Some(explorer) = explorers.current() {
			request = send_request(stream, client, RealmsProtocol::Explorer(Move::ChangeRegion(realm_id, region.id, explorer.id)));
		}
	}

	request
}