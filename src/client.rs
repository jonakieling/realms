
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
	ExplorerOrders,
	Realms,
	ExplorerMove,
	ExplorerActions,
	ExplorerInventory,
	Particularities
}

#[derive(Debug)]
pub struct Data {
	pub id: ClientId,
	pub realm: Realm,
	pub realms: SelectionStorage<RealmId>,
	pub explorer_orders: SelectionStorage<ExplorerOrders>,
	pub active: InteractiveUi
}

#[derive(Debug, Clone)]
pub enum ExplorerOrders {
	Inventory,
	Actions,
	Embark,
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
			realms = response_realms;
		}

		// init realm, beware that id 0 is a valid id so this might geht overriden by the server side realm with id 0
		let realm = Realm::plain(0);

		Periscope {
			stream,
			data: Data {
				id: client_id,
				realm,
				realms,
				explorer_orders: SelectionStorage::new_from(&vec![ExplorerOrders::Inventory, ExplorerOrders::Move]),
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
	let mut buffer = [0; 2024];
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
			    InteractiveUi::ExplorerOrders => {
			    	handle_explorer_orders_events(stream, data, key);
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
			    },
			    InteractiveUi::Particularities => {
			    	handle_particularities_events(stream, data, key);
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
				data.realm = response_realm;
			}
			if let RealmsProtocol::RealmsList(response_realms) = send_request(stream, data.id, RealmsProtocol::RequestRealmsList) {
				data.realms = response_realms;
			}
			data.realms.last();
		},
		event::Key::Char('\n') => {
    		let realm_id = data.realms.current().expect("could not access current realm selection.");
    		let current_realm_id = data.realm.id;
    		if *realm_id != current_realm_id || data.realm.island.regions.iter().len() == 0 {
	    		if let RealmsProtocol::Realm(response_realm) = send_request(stream, data.id, RealmsProtocol::RequestRealm(*realm_id)) {
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
	    	data.realm.island.regions.prev();
		},
		event::Key::Down => {
	    	data.realm.island.regions.next();
		},
		event::Key::Right => {
	    	data.active = InteractiveUi::Explorers;
        	sync_regions_with_explorer(data);
			update_explorer_available_orders(data);
		},
		event::Key::Left => {
	    	data.active = InteractiveUi::Particularities;
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
	    	data.realm.expedition.explorers.prev();
        	sync_regions_with_explorer(data);
			update_explorer_available_orders(data);
		},
		event::Key::Down => {
	    	data.realm.expedition.explorers.next();
	    	update_explorer_available_orders(data);
        	sync_regions_with_explorer(data);
		},
		event::Key::Right | event::Key::Char('\n') => {
	    	data.active = InteractiveUi::ExplorerOrders;
		},
		event::Key::Left => {
	    	data.active = InteractiveUi::Regions;
	    	data.realm.island.regions.at(0);
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

fn handle_explorer_orders_events(_stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.explorer_orders.prev();
		},
		event::Key::Down => {
	    	data.explorer_orders.next();
		},
		event::Key::Left | event::Key::Esc => {
	    	data.active = InteractiveUi::Explorers;
			update_explorer_available_orders(data);
		},
		event::Key::Right => {
	    	data.active = InteractiveUi::Particularities;
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		event::Key::Char('\n') => {
			match data.explorer_orders.current() {
			    Some(ExplorerOrders::Inventory) => {
		        	data.active = InteractiveUi::ExplorerInventory;
			    },
			    Some(ExplorerOrders::Actions) => data.active = InteractiveUi::ExplorerActions,
			    Some(ExplorerOrders::Move) | Some(ExplorerOrders::Embark) => data.active = InteractiveUi::ExplorerMove,
			    None => {
			    	data.active = InteractiveUi::Explorers;
	    			update_explorer_available_orders(data);
			    }
			};
		},
		_ => { }
	}
}

fn handle_explorer_move_events(stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.realm.island.regions.prev();
		},
		event::Key::Down => {
	    	data.realm.island.regions.next();
		},
		event::Key::Esc => {
	    	data.active = InteractiveUi::ExplorerOrders;
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		event::Key::Char('\n') => {
    		{
				// request reset regions index, we set it back afterwards
				let last_index = data.realm.island.regions.current_index();
				let last_explorers_index = data.realm.expedition.explorers.current_index();

	    		if let RealmsProtocol::Realm(response_realm) = explorer_move(stream, data.id, data.realm.id, &mut data.realm.island.regions, &mut data.realm.expedition.explorers) {
		    		data.realm = response_realm;

	    			update_explorer_available_orders(data);
				}

				data.realm.island.regions.at(last_index);
				data.realm.expedition.explorers.at(last_explorers_index);
    		}
        	sync_regions_with_explorer(data);
	    	data.active = InteractiveUi::ExplorerOrders;
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
	    	data.active = InteractiveUi::ExplorerOrders;
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		event::Key::Char('\n') => {
    		{
				// request reset regions and explorers index, we set it back afterwards
				let last_region_index = data.realm.island.regions.current_index();
				let last_explorers_index = data.realm.expedition.explorers.current_index();

	    		if let RealmsProtocol::Realm(response_realm) = explorer_action(stream, data.id, data.realm.id, &mut data.realm.island.regions, &mut data.realm.expedition.explorers) {
		    		data.realm = response_realm;
				}

				data.realm.island.regions.at(last_region_index);
				data.realm.expedition.explorers.at(last_explorers_index);
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
	    	data.active = InteractiveUi::ExplorerOrders;
		},
		event::Key::Char('l') => {
			data.active = InteractiveUi::Realms;
		},
		event::Key::Char('\n') => {
		},
		_ => { }
	}
}

fn handle_particularities_events(_stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
			if let Some(region) = data.realm.island.regions.current_mut() {
			    region.particularities.prev();
			}
		},
		event::Key::Down => {
			if let Some(region) = data.realm.island.regions.current_mut() {
			    region.particularities.next();
			}
		},
		event::Key::Left => {
	    	data.active = InteractiveUi::ExplorerOrders;
			update_explorer_available_orders(data);
		},
		event::Key::Right => {
	    	data.active = InteractiveUi::Regions;
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
	if let Some(explorer_region) = data.realm.expedition.explorers.current().expect("could not access current explorers selection.").region {
		current_explorer_region = explorer_region;
	}
	let mut current_explorer_region_index = 0;
	for (index, region) in data.realm.island.regions.iter().enumerate() {
	    if current_explorer_region == region.id {
	        current_explorer_region_index = index;
	    }
	}
	data.realm.island.regions.at(current_explorer_region_index);
}

fn update_explorer_available_orders(data: &mut Data) {
	if let Some(explorer) = data.realm.expedition.explorers.current() {
	    if explorer.region.is_some() {
	    	data.explorer_orders = SelectionStorage::new_from(&vec![ExplorerOrders::Inventory, ExplorerOrders::Actions, ExplorerOrders::Move]);
	    } else {
	    	data.explorer_orders = SelectionStorage::new_from(&vec![ExplorerOrders::Inventory, ExplorerOrders::Embark]);
	    }
	}
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