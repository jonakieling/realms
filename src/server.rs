
use std::collections::HashMap;
use std::net::TcpStream;
use std::net::TcpListener;
use std::net::Shutdown;
use std::io::prelude::*;
use std::thread;
use std::sync::{Mutex, Arc, mpsc};
use std::io;

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;

use chrono::{Local, DateTime};

use uuid::Uuid;

use tokens::*;
use utility::*;
use realms::*;
use server_dashboard::*;

pub struct Universe {
	pub realms: Vec<RealmStrategy>,
	pub clients: HashMap<Uuid, Client>,
	pub requests: Vec<(ClientId, RealmsProtocol, DateTime<Local>)>
}

#[derive(Debug, Clone)]
pub struct Client {
	pub id: Uuid,
	pub connected: bool,
	pub time: DateTime<Local>,
	pub realms_list: SelectionStorage<RealmId>,
	pub completed_variants: Vec<RealmVariant>
}

impl Client {
	pub fn new(id: ClientId) -> Client {
		Client {
			id,
			connected: true,
			time: Local::now(),
			realms_list: SelectionStorage::new(),
			completed_variants: vec![]
		}
	}
}

pub fn run(host: String) {
	// channel to notify ui to update
    let (tx, rx) = mpsc::channel();
    // global state of all games and clients
	let universe = Arc::new(Mutex::new(Universe { realms: vec![], clients: HashMap::new(), requests: vec![] }));
    // getting arc of universe for this thread (ui) before moved to server thread
    let ui_glimpse = Arc::clone(&universe);

    let listener = TcpListener::bind(&host).expect(&format!("could not bind tcp listener to {}", host));
    // server thread
	let _server = thread::spawn(move || {
		// client threads
		for stream in listener.incoming() {
		    let mut glimpse = Arc::clone(&universe);
		    let client_tx = tx.clone();
			thread::spawn(move || {

				let mut stream = stream.expect("could not get tcp stream.");
				loop {
				    let mut buffer = [0; 4096];

				    stream.read(&mut buffer).expect("could not read request into buffer.");
				    stream.flush().expect("could not flush request stream.");

				    let (client_id, request): (ClientId, RealmsProtocol) = deserialize(&buffer).expect("could not deserialize client request.");

				    let mut lock_glimpse = glimpse.lock().unwrap();

				    // fetch current client if any
				    let current_client: Option<Client> = lock_glimpse.clients.get_mut(&client_id).cloned();

				    // seperate client and no-client request handling
		        	if let Some(mut client) = current_client {
		        		let response = handle_request(&mut lock_glimpse.realms, &mut client, request.clone());
		        	    send_response(&response, &stream).expect("sending response failed.");

		        		let disconnect = !client.connected;

		        		// log request
				    	lock_glimpse.requests.push((client.id, request, Local::now()));
		        		// update client in list
		        		lock_glimpse.clients.insert(client.id, client.clone());
		        		
		        	    if disconnect {
							stream.shutdown(Shutdown::Both).expect("stream could not shut down.");
				    		client_tx.send(Some(0)).unwrap();
		        	    	break;
		        	    }
		        	} else {
		        	    let response = handle_connecting_requests(&mut lock_glimpse.clients, request);
		        	    send_response(&response, &stream).expect("sending response failed.");
		        	}

				    client_tx.send(Some(1)).unwrap();
				}
			});
	    }
	});

    // tui terminal
    let backend = RawBackend::new().unwrap();
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();
    terminal.hide_cursor().unwrap();
    {
		let lock_glimpse = ui_glimpse.lock().unwrap();
		draw(&mut terminal, &lock_glimpse.requests, &lock_glimpse.clients, &lock_glimpse.realms).expect("ui could not be drawn.");
    }
	loop {
		let _request = rx.recv().unwrap();
		let lock_glimpse = ui_glimpse.lock().unwrap();
		draw(&mut terminal, &lock_glimpse.requests, &lock_glimpse.clients, &lock_glimpse.realms).expect("ui could not be drawn.");
	}
}

fn handle_connecting_requests(clients: &mut HashMap<Uuid, Client>, request: RealmsProtocol) -> RealmsProtocol {
	match request {
		// a connect request when the server could not find the id matching client acts as a register request.
        RealmsProtocol::Register | RealmsProtocol::Connect(_) => {
        	let id = Uuid::new_v4();
			clients.insert(id, Client::new(id));

    		RealmsProtocol::Connect(id)
        },
        _ => {
			RealmsProtocol::Void
		}
	}
}

fn handle_request(realm_strategies: &mut Vec<RealmStrategy>, client: &mut Client, request: RealmsProtocol) -> RealmsProtocol {
	client.time = Local::now();

	match request {
		RealmsProtocol::Connect(id) => {
			client.connected = true;

    		RealmsProtocol::Connect(id)
        },
        RealmsProtocol::RequestRealmsList => {
    		RealmsProtocol::RealmsList(client.realms_list.clone())
        },
        RealmsProtocol::RequestNewRealm => {
        	let id = realm_strategies.len();
	        let mut strategy = RealmStrategy::new(id, RealmVariant::Tutorial);
	        let realm = strategy.realm.clone();
	        let realm_id = strategy.realm.id;
    		realm_strategies.push(strategy);
    		client.realms_list.insert(realm_id);
    		RealmsProtocol::Realm(realm)
        },
        RealmsProtocol::RequestRealm(realm_id) => {
        	if realm_strategies.len() > realm_id {
        	    if let Some(RealmStrategy {variant: _, realm, template: _}) = realm_strategies.get_mut(realm_id) {
					RealmsProtocol::Realm(realm.clone())
        	    } else {
					RealmsProtocol::Void
        	    }
        	} else {
        		// send new realm on miss
	        	let id = realm_strategies.len();
		        let mut strategy = RealmStrategy::new(id, RealmVariant::Tutorial);
		        let realm = strategy.realm.clone();
		        let realm_id = strategy.realm.id;
	    		realm_strategies.push(strategy);
	    		client.realms_list.insert(realm_id);
	    		RealmsProtocol::Realm(realm)
        	}
        },
        RealmsProtocol::Explorer(Move::ChangeRegion(realm_id, region_id, explorer_id)) => {
        	// todo: check consequences of move for realm template

        	let mut valid_move = false;
        	if let Some(strategy) = realm_strategies.get(realm_id) {
        	    valid_move = strategy.valid_move(explorer_id, region_id);
        	}

        	if valid_move {
	        	if let Some(explorer) = realm_strategies.get_mut(realm_id).explorer(explorer_id) {
	    	    	explorer.region = Some(region_id);
	        	}

	        	if let Some(strategy) = realm_strategies.get_mut(realm_id) {
	        		let done_before = strategy.realm.done;
	    	    	strategy.state();
	    	    	if strategy.realm.done && !done_before {
	    	    		client.completed_variants.push(strategy.variant.clone());
	    	    	}

					RealmsProtocol::Realm(strategy.realm.clone())
	        	} else {
					RealmsProtocol::Void
	    	    }
        	} else {
				RealmsProtocol::Void
        	}
        },
        RealmsProtocol::Explorer(Move::Action(realm_id, region_id, explorer_id, action)) => {
        	// todo: save result of action in realm template aswell
        	// or only in template and let client side realm be filled by state update

        	let mut valid_action = false;
        	if let Some(strategy) = realm_strategies.get(realm_id) {
        	    valid_action = strategy.valid_action(explorer_id, region_id, &action);
        	}

			valid_action &= realm_strategies.get_mut(realm_id).region_explorer(region_id, explorer_id).is_some();
			let mut region_to_update: Option<Region> = None;
			if let Some(mut region) = realm_strategies.get_mut(realm_id).region(region_id) {
			    if valid_action {
    				match action {
    				    ExplorerAction::Build => {
    				    	region.buildings.insert("\u{2302}".to_string());
    				    },
    				    ExplorerAction::Map => {
    				    	region.mapped = true;
    				    },
    				    ExplorerAction::Hunt => {
    				    	if region.resources > 0 {
    				    		region.resources -= 1;
    				    	} else {
    				    		valid_action = false;
    				    	}
    				    },
    				    ExplorerAction::Sail => {},
    				    ExplorerAction::Wait => {}
    				}
    				region_to_update = Some(region.clone());
    	    	}
			}

        	if let Some(strategy) = realm_strategies.get_mut(realm_id) {
        		if valid_action {
        			if let Some(region) = region_to_update {
						strategy.template.regions.insert(region.id, region);
        			}
        		}
        	}

			if let Some(strategy) = realm_strategies.get_mut(realm_id) {
	        	if valid_action {
	        		let done_before = strategy.realm.done;
	    	    	strategy.state();
	    	    	if strategy.realm.done && !done_before {
	    	    		client.completed_variants.push(strategy.variant.clone());
	    	    	}

					RealmsProtocol::Realm(strategy.realm.clone())

	        	} else {
					RealmsProtocol::Void
	        	}
			} else {
				RealmsProtocol::Void
    	    }
        },
        RealmsProtocol::DropEquipment(realm_id, _, explorer_id, item) => {
        	// todo: drop item in realm template aswell

        	if let Some(region) = realm_strategies.get_mut(realm_id).explorer_region(explorer_id) {
        	    region.particularities.insert(Particularity::Item(item));
        	}
        	if let Some(explorer) = realm_strategies.get_mut(realm_id).explorer(explorer_id) {
        	    explorer.inventory.storage_mut().iter().position(move |ref n| **n == ExplorerItem::Equipment(item)).map(|equipment| {
    				explorer.inventory.storage_mut().remove(equipment);
        		});
        	}

			if let Some(RealmStrategy {variant: _, ref mut realm, template: _}) = realm_strategies.get_mut(realm_id) {
				RealmsProtocol::Realm(realm.clone())
		    } else {
		    	RealmsProtocol::Void
		    }
        },
        RealmsProtocol::PickEquipment(realm_id, region_id, explorer_id, item) => {
        	// todo: pick item in realm template aswell

        	if let Some(region) = realm_strategies.get_mut(realm_id).explorer_region(explorer_id) {
        		region.particularities.storage_mut().iter().position(move |ref n| **n == Particularity::Item(item)).map(|equipment| {
    				region.particularities.storage_mut().remove(equipment);
        		});
        	}
        	if let Some(explorer) = realm_strategies.get_mut(realm_id).region_explorer(region_id, explorer_id) {
        	    explorer.inventory.insert(ExplorerItem::Equipment(item));
        	}

			if let Some(RealmStrategy {variant: _, ref mut realm, template: _}) = realm_strategies.get_mut(realm_id) {
				RealmsProtocol::Realm(realm.clone())
		    } else {
		    	RealmsProtocol::Void
		    }
        },
        RealmsProtocol::ForgetParticularity(realm_id, region_id, explorer_id, particularity) => {
		    if let Some(explorer) = realm_strategies.get_mut(realm_id).explorer(explorer_id) {
        	    explorer.inventory.storage_mut().iter().position(move |ref n| **n == ExplorerItem::Particularity(region_id, particularity)).map(|equipment| {
    				explorer.inventory.storage_mut().remove(equipment);
        		});
        	}
        	
		    if let Some(RealmStrategy {variant: _, ref mut realm, template: _}) = realm_strategies.get_mut(realm_id) {
				RealmsProtocol::Realm(realm.clone())
			} else {
				RealmsProtocol::Void
    	    }
        },
        RealmsProtocol::InvestigateParticularity(realm_id, region_id, explorer_id, item) => {
        	if let Some(explorer) = realm_strategies.get_mut(realm_id).region_explorer(region_id, explorer_id) {
        	    explorer.inventory.insert(ExplorerItem::Particularity(region_id, item));
        	}

		    if let Some(RealmStrategy {variant: _, ref mut realm, template: _}) = realm_strategies.get_mut(realm_id) {
				RealmsProtocol::Realm(realm.clone())
			} else {
				RealmsProtocol::Void
    	    }
        },
        RealmsProtocol::Quit => {
	    	client.connected = false;

			RealmsProtocol::Quit
        },
        _ => { RealmsProtocol::Void }
    }
}

fn send_response(data: &RealmsProtocol, mut stream: &TcpStream) -> Result<(), io::Error> {
	let raw = serialize(data).expect("could not serialize data for response.");
	stream.write(&raw).expect("could not write to tcp stream.");
	stream.flush().expect("could not flush response stream.");
	Ok(())
}
