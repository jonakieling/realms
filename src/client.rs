
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
	ExplorerSelect,
	Realms,
	ExplorerMove,
	ExplorerActions,
	ExplorerInventory
}

#[derive(Debug)]
pub struct Data {
	pub id: ClientId,
	pub realm: Realm,
	pub realms: SelectionStorage<RealmId>,
	pub regions: SelectionStorage<Region>,
	pub explorers: SelectionStorage<Explorer>,
	pub explorer_select: SelectionStorage<ExplorerSelect>,
	pub explorer_inventory: SelectionStorage<Gear>,
	pub active: InteractiveUi
}

#[derive(Debug, Clone)]
pub enum ExplorerSelect {
	Inventory,
	Actions,
	Move
}

pub struct Periscope {
	pub stream: TcpStream,
	pub data: Data
}

impl Periscope {
	pub fn new(mut stream: TcpStream) -> Periscope {
		let mut client_id = 0;
		if let RealmsProtocol::Connect(id) = send_request(&mut stream, 0, RealmsProtocol::Register) {
			client_id = id;
		}

		let mut realms = SelectionStorage::new();
		if let RealmsProtocol::RealmsList(response_realms) = send_request(&mut stream, client_id, RealmsProtocol::RequestRealmsList) {
			realms = SelectionStorage::new_from(&response_realms);
		}

		// init realm, beware that id 0 is a valid id so this might geht overriden by the server side realm with id 0
		let realm = Realm::new(0);

		Periscope {
			stream,
			data: Data {
				id: client_id,
				realm,
				realms,
				regions: SelectionStorage::new(),
				explorers: SelectionStorage::new(),
				explorer_select: SelectionStorage::new_from(&vec![ExplorerSelect::Inventory, ExplorerSelect::Actions, ExplorerSelect::Move]),
				explorer_inventory: SelectionStorage::new(),
				active: InteractiveUi::Realms
			}
		}
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
			    InteractiveUi::ExplorerSelect => {
			    	handle_explorer_select_events(stream, data, key);
			    },
			    InteractiveUi::Realms => {
			    	handle_realms_events(stream, data, key);
			    },
			    InteractiveUi::ExplorerMove => {
			    	handle_explorer_move_events(stream, data, key);
			    },
			    InteractiveUi::ExplorerActions => {
			    	handle_explorer_actions_events(stream, data, key);
			    },
			    InteractiveUi::ExplorerInventory => {
			    	handle_explorer_inventory_events(stream, data, key);
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
				data.realm = response_realm;
			}
			if let RealmsProtocol::RealmsList(response_realms) = send_request(stream, data.id, RealmsProtocol::RequestRealmsList) {
				data.realms = SelectionStorage::new_from(&response_realms);
			}
			data.realms.last();
		},
		event::Key::Char('\n') => {
    		let realm_id = data.realms.current().expect("could not access current realm selection.");
    		let current_realm_id = data.realm.id;
    		if *realm_id != current_realm_id {
	    		if let RealmsProtocol::Realm(response_realm) = send_request(stream, data.id, RealmsProtocol::RequestRealm(*realm_id)) {
					data.regions =  SelectionStorage::new_from(&response_realm.island.regions);
		    		data.explorers = SelectionStorage::new_from(&response_realm.expedition.explorers);
					data.realm = response_realm;
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
	    	data.active = InteractiveUi::ExplorerSelect;
        	sync_regions_with_explorer(data);
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		_ => { }
	}
}

fn handle_explorer_events(_stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.explorers.prev();
        	sync_regions_with_explorer(data);
		},
		event::Key::Down => {
	    	data.explorers.next();
        	sync_regions_with_explorer(data);
		},
		event::Key::Right | event::Key::Char('\n') => {
	    	data.active = InteractiveUi::ExplorerSelect;
		},
		event::Key::Left => {
	    	data.active = InteractiveUi::Regions;
	    	data.regions.at(0);
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		event::Key::Char('a') => {
			data.active = InteractiveUi::ExplorerActions;
		},
		event::Key::Char('i') => {
			data.active = InteractiveUi::ExplorerInventory;
		},
		event::Key::Char('m') => {
			data.active = InteractiveUi::ExplorerMove;
		},
		_ => { }
	}
}

fn handle_explorer_select_events(_stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.explorer_select.prev();
		},
		event::Key::Down => {
	    	data.explorer_select.next();
		},
		event::Key::Left => {
	    	data.active = InteractiveUi::Explorers;
		},
		event::Key::Right => {
	    	data.active = InteractiveUi::Regions;
	    	data.regions.at(0);
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		event::Key::Char('\n') => {
			match data.explorer_select.current() {
			    Some(ExplorerSelect::Inventory) => {
		        	if let Some(ref explorer) = &data.explorers.current() {
		        	    data.explorer_inventory = SelectionStorage::new_from(&explorer.inventory);
		        	}
		        	data.active = InteractiveUi::ExplorerInventory;
			    },
			    Some(ExplorerSelect::Actions) => data.active = InteractiveUi::ExplorerActions,
			    Some(ExplorerSelect::Move) => data.active = InteractiveUi::ExplorerMove,
			    None => data.active = InteractiveUi::Explorers
			};
		},
		_ => { }
	}
}

fn handle_explorer_move_events(stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.regions.prev();
		},
		event::Key::Down => {
	    	data.regions.next();
		},
		event::Key::Esc => {
	    	data.active = InteractiveUi::ExplorerSelect;
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
					data.realm = response_realm;
				}

				data.regions.at(last_index);
				data.explorers.at(last_explorers_index);
    		}
        	sync_regions_with_explorer(data);
	    	data.active = InteractiveUi::ExplorerSelect;
		},
		_ => { }
	}
}

fn handle_explorer_actions_events(stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
		},
		event::Key::Down => {
		},
		event::Key::Esc => {
	    	data.active = InteractiveUi::ExplorerSelect;
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
					data.realm = response_realm;
				}

				data.regions.at(last_region_index);
				data.explorers.at(last_explorers_index);
    		}
        	sync_regions_with_explorer(data);
		},
		_ => { }
	}
}

fn handle_explorer_inventory_events(_stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
		},
		event::Key::Down => {
		},
		event::Key::Esc => {
	    	data.active = InteractiveUi::ExplorerSelect;
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		event::Key::Char('\n') => {
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