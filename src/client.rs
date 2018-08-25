
use std::fs::File;
use std::net::{TcpStream, Shutdown};
use std::sync::mpsc::Receiver;
use std::io::prelude::*;
use std::io;

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;
use termion::event;

use uuid::Uuid;

use Event;
use tokens::*;
use utility::*;

use client_dashboard::draw;

#[derive(Debug)]
pub enum InteractiveUi {
	Explorers,
	ExplorerOrders,
	Realms,
	Regions,
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
	pub active: InteractiveUi,
	pub tabs: SelectionStorage<String>
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

		let mut client_id: Uuid = Uuid::new_v4();
    	if let Ok(mut file) = File::open("client.id") {
			let mut stored_client_id = String::new();
		    file.read_to_string(&mut stored_client_id).expect("could not read contents of file client.id");
		    client_id = Uuid::parse_str(&stored_client_id).expect("could not parse stored client id as usize.");

		    // try to connect previous client
		    if let RealmsProtocol::Connect(id) = send_request(&mut stream, client_id, RealmsProtocol::Connect(client_id)) {
				client_id = id;
			}
    	} else {
    		// register new client
			if let RealmsProtocol::Connect(id) = send_request(&mut stream, client_id, RealmsProtocol::Register) {
				client_id = id;
			}
    	}

		File::create("client.id").expect("could not create file client.id").write_fmt(format_args!("{}", client_id)).expect("could not write to file client.id");

		// init realm, should get overriden by the server
		let mut realm = Realm::new(0);
		if let RealmsProtocol::Realm(response_realm) = send_request(&mut stream, client_id, RealmsProtocol::RequestNewRealm) {
			realm = response_realm;
		}
		
		let mut realms = SelectionStorage::new();
		if let RealmsProtocol::RealmsList(response_realms) = send_request(&mut stream, client_id, RealmsProtocol::RequestRealmsList) {
			realms = response_realms;
		}

		realms.last();

		let mut periscope = Periscope {
			stream,
			data: Data {
				id: client_id,
				realm,
				realms,
				explorer_orders: SelectionStorage::new(),
				active: InteractiveUi::Explorers,
				tabs: SelectionStorage::new_from(&vec!["Current Realm".to_string(), "Realms".to_string()])
			}
		};

		update_explorer_available_orders(&mut periscope.data);

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
	let mut buffer = [0; 4096];
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
	    	match key {
	    	    event::Key::Char('\t') => {
	    	    	data.tabs.next();
	    	    	match data.tabs.current_index() {
		                0 => {
		                    data.active = InteractiveUi::Explorers;
		                },
		                1 => {
		                    data.active = InteractiveUi::Realms;
		                },
		                _ => {
		                    data.active = InteractiveUi::Regions;
		                }
		            }
	    	    },
	    	    event::Key::Char('q') => {
					if let RealmsProtocol::Quit = send_request(stream, data.id, RealmsProtocol::Quit) {
						stream.shutdown(Shutdown::Both).expect("connection should have terminated.");
						should_continue = false;
					}
		    	},
		    	_ => {
					match data.active {
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
					    },
					    InteractiveUi::Regions => {
					    	handle_regions_events(stream, data, key);
					    }
					}
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
    		{
    			let realm_id = data.realms.current().expect("could not access current realm selection.");
	    		let current_realm_id = data.realm.id;
	    		if *realm_id != current_realm_id || data.realm.island.regions.iter().len() == 0 {
		    		if let RealmsProtocol::Realm(response_realm) = send_request(stream, data.id, RealmsProtocol::RequestRealm(*realm_id)) {
						data.realm = response_realm;
					}
	    		}
		    	data.active = InteractiveUi::Explorers;
		    	data.tabs.next();
    		}
		},
		_ => { }
	}
}

fn handle_explorer_events(_stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
	    	data.realm.expedition.explorers.prev();
			update_explorer_available_orders(data);
		},
		event::Key::Down => {
	    	data.realm.expedition.explorers.next();
	    	update_explorer_available_orders(data);
		},
		event::Key::Right | event::Key::Char('\n') => {
	    	data.active = InteractiveUi::ExplorerOrders;
		},
		event::Key::Left => {
	    	data.active = InteractiveUi::Explorers;
	    	data.realm.island.regions.at(0);
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
		event::Key::Left | event::Key::Backspace => {
	    	data.active = InteractiveUi::Explorers;
			update_explorer_available_orders(data);
		},
		event::Key::Right => {
	    	data.active = InteractiveUi::Particularities;
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
		event::Key::Backspace => {
	    	data.active = InteractiveUi::ExplorerOrders;
		},
		event::Key::Char('\n') => {
    		{
				// request reset explorers index, we set it back afterwards
				let last_explorers_index = data.realm.expedition.explorers.current_index();
	    		
	    		if let RealmsProtocol::Realm(response_realm) = explorer_move(stream, data.id, data.realm.id, &mut data.realm.island.regions, &mut data.realm.expedition.explorers) {
		    		data.realm = response_realm;
				}
				data.realm.expedition.explorers.at(last_explorers_index);
    		}
	    	data.active = InteractiveUi::ExplorerOrders;

			update_explorer_available_orders(data);
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
		event::Key::Backspace => {
	    	data.active = InteractiveUi::ExplorerOrders;
		},
		event::Key::Char('\n') => {
    		{
				// request reset explorers index, we set it back afterwards
				let last_explorers_index = data.realm.expedition.explorers.current_index();

	    		if let RealmsProtocol::Realm(response_realm) = explorer_action(stream, data.id, data.realm.id, &mut data.realm.island.regions, &mut data.realm.expedition.explorers) {
		    		data.realm = response_realm;
				}
				data.realm.expedition.explorers.at(last_explorers_index);
    		}
        	sync_regions_with_explorer(data);
		},
		_ => { }
	}
}

fn handle_explorer_inventory_events(stream: &mut TcpStream, data: &mut Data, key: event::Key) {
	match key {
		event::Key::Up => {
			if let Some(ref mut explorer) = data.realm.expedition.explorers.current_mut() {
			    explorer.inventory.prev();
			}
		},
		event::Key::Down => {
			if let Some(ref mut explorer) = data.realm.expedition.explorers.current_mut() {
			    explorer.inventory.next();
			}
		},
		event::Key::Char('\n') => {
			let last_explorers_index = data.realm.expedition.explorers.current_index();
    		if let RealmsProtocol::Realm(response_realm) = explorer_drop(stream, data.id, data.realm.id, &mut data.realm.island.regions, &mut data.realm.expedition.explorers) {
	    		data.realm = response_realm;
			}
			data.realm.expedition.explorers.at(last_explorers_index);
    		sync_regions_with_explorer(data);
		},
		event::Key::Backspace => {
	    	data.active = InteractiveUi::ExplorerOrders;
		},
		_ => { }
	}
}

fn handle_particularities_events(stream: &mut TcpStream, data: &mut Data, key: event::Key) {
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
		event::Key::Char('\n') => {
			let last_explorers_index = data.realm.expedition.explorers.current_index();
			if let RealmsProtocol::Realm(response_realm) = explorer_handle_particularity(stream, data.id, data.realm.id, &mut data.realm.island.regions, &mut data.realm.expedition.explorers) {
	    		data.realm = response_realm;
			}
			data.realm.expedition.explorers.at(last_explorers_index);
    		sync_regions_with_explorer(data);
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
		_ => { }
	}
}

fn sync_regions_with_explorer(data: &mut Data) {
	if let Some(explorer_region) = data.realm.expedition.explorers.current().expect("could not access current explorers selection.").region {
		data.realm.island.regions.at(explorer_region);
	}
}

fn update_explorer_available_orders(data: &mut Data) {

	sync_regions_with_explorer(data);

	if let Some(explorer) = data.realm.expedition.explorers.current() {
	    if explorer.region.is_some() {
	    	data.explorer_orders = SelectionStorage::new_from(&vec![ExplorerOrders::Inventory, ExplorerOrders::Actions, ExplorerOrders::Move]);
	    } else {
	    	data.explorer_orders = SelectionStorage::new_from(&vec![ExplorerOrders::Inventory, ExplorerOrders::Embark]);
	    }
	}
}

fn explorer_action(stream: &mut TcpStream, client: ClientId, realm_id: RealmId, regions: &mut SelectionHashMap<Region>, explorers: &mut SelectionStorage<Explorer>) -> RealmsProtocol {
	let mut request = RealmsProtocol::Void;

	if let Some(region) = regions.current() {
		if let Some(explorer) = explorers.current() {
			if let Some(action) = explorer.trait_actions().first() {
				request = send_request(stream, client, RealmsProtocol::Explorer(Move::Action(realm_id, region.id, explorer.id, action.clone())));
			}
		}
	}

	request
}

fn explorer_drop(stream: &mut TcpStream, client: ClientId, realm_id: RealmId, regions: &mut SelectionHashMap<Region>, explorers: &mut SelectionStorage<Explorer>) -> RealmsProtocol {
	let mut request = RealmsProtocol::Void;

	if let Some(region) = regions.current() {
		if let Some(explorer) = explorers.current() {
			match explorer.inventory.current() {
			    Some(ExplorerItem::Equipment(item)) => {
			    	request = send_request(stream, client, RealmsProtocol::DropEquipment(realm_id, region.id, explorer.id, item.clone()));
			    },
			    Some(ExplorerItem::Particularity(region_id, particularity)) => {
			    	request = send_request(stream, client, RealmsProtocol::ForgetParticularity(realm_id, *region_id, explorer.id, particularity.clone()));	
			    },
			    _ => { }
			}
		}
	}

	request
}

fn explorer_handle_particularity(stream: &mut TcpStream, client: ClientId, realm_id: RealmId, regions: &mut SelectionHashMap<Region>, explorers: &mut SelectionStorage<Explorer>) -> RealmsProtocol {
	let mut request = RealmsProtocol::Void;

	if let Some(region) = regions.current() {
		if let Some(explorer) = explorers.current() {
			match region.particularities.current() {
			    Some(Particularity::Item(item)) => {
			    	request = send_request(stream, client, RealmsProtocol::PickEquipment(realm_id, region.id, explorer.id, item.clone()));
			    },
			    Some(particularity) => {
			    	request = send_request(stream, client, RealmsProtocol::InvestigateParticularity(realm_id, region.id, explorer.id, particularity.clone()));
			    },
			    _ => { }
			}
		}
	}

	request
}

fn explorer_move(stream: &mut TcpStream, client: ClientId, realm_id: RealmId, regions: &mut SelectionHashMap<Region>, explorers: &mut SelectionStorage<Explorer>) -> RealmsProtocol {
	let mut request = RealmsProtocol::Void;

	if let Some(region) = regions.current() {
		if let Some(explorer) = explorers.current() {
			request = send_request(stream, client, RealmsProtocol::Explorer(Move::ChangeRegion(realm_id, region.id, explorer.id)));
		}
	}

	request
}