
use std::collections::HashMap;
use std::net::TcpStream;
use std::net::TcpListener;
use std::net::Shutdown;
use std::io::prelude::*;
use std::io;

use bincode::{serialize, deserialize};

use tui::Terminal;
use tui::backend::RawBackend;

use chrono::{Local, DateTime};

use uuid::Uuid;

use tokens::*;
use utility::*;
use realms::*;
use server_dashboard::draw;

pub struct Universe {
	pub listener: TcpListener,
	pub realms: Vec<Realm>,
	pub requests: Vec<(ClientId, RealmsProtocol, DateTime<Local>)>,
	pub clients: HashMap<Uuid, Client>
}

#[derive(Debug, Clone)]
pub struct Client {
	pub id: Uuid,
	pub connected: bool,
	pub time: DateTime<Local>,
	pub realm_variant: RealmVariant
}

impl Client {
	pub fn new(id: ClientId) -> Client {
		Client {
			id,
			connected: true,
			time: Local::now(),
			realm_variant: RealmVariant::Tutorial
		}
	}
}

impl Universe {
	pub fn run(mut self, t: &mut Terminal<RawBackend>) -> Result<(), io::Error> {
	    draw(t, &self.requests, &self.clients, &self.realms)?;
	    for stream in self.listener.incoming() {
			let mut stream = stream.expect("could not get tcp stream.");
			loop {
			    let mut buffer = [0; 2024];

			    stream.read(&mut buffer).expect("could not read request into buffer.");
			    stream.flush().expect("could not flush request stream.");

			    let (client_id, request): (ClientId, RealmsProtocol) = deserialize(&buffer).expect("could not deserialize client request.");

			    // fetch current client if any
			    let mut current_client: Option<Client> = self.clients.get_mut(&client_id).cloned();

			    // seperate client and no-client request handling
	        	if let Some(mut client) = current_client {
	        	    handle_request(&mut self.requests, &mut self.realms, &mut stream, &mut client, request)?;
	        		let disconnect = !client.connected;
	        		self.clients.insert(client.id, client);

	        	    if disconnect {
	        	    	// draw update before client exits
			    		draw(t, &self.requests, &self.clients, &self.realms)?;
	        	    	break;
	        	    }
	        	} else {
	        	    handle_connecting_requests(&mut self.clients, &mut self.requests, &mut stream, request)?;
	        	}


	        	// draw dashboard after requests and responses have been handled
			    draw(t, &self.requests, &self.clients, &self.realms)?;
			}
	    }

	    // reset terminal before exiting
	    t.show_cursor().unwrap();
	    t.clear().unwrap();

	    Ok(())
	}
}

fn handle_connecting_requests(clients: &mut HashMap<Uuid, Client>, requests: &mut Vec<(ClientId, RealmsProtocol, DateTime<Local>)>, stream: &mut TcpStream, request: RealmsProtocol) -> Result<(), io::Error> {
	match request {
		// a connect request when the server could not find the id matching client acts as a register request.
        RealmsProtocol::Register | RealmsProtocol::Connect(_) => {
        	let id = Uuid::new_v4();
			send_response(&RealmsProtocol::Connect(id), &stream)?;
			clients.insert(id, Client::new(id));
    		requests.push((id, request, Local::now()));
        },
        _ => {
			send_response(&RealmsProtocol::Void, &stream)?;
		}
	}

	Ok(())
}

fn handle_request(requests: &mut Vec<(ClientId, RealmsProtocol, DateTime<Local>)>, realms: &mut Vec<Realm>, stream: &mut TcpStream, client: &mut Client, request: RealmsProtocol) -> Result<(), io::Error> {
	let client_id = client.id;
	client.time = Local::now();

	match request {
		RealmsProtocol::Connect(id) => {
			send_response(&RealmsProtocol::Connect(id), &stream)?;
			client.connected = true;
    		requests.push((id, request, Local::now()));
        },
        RealmsProtocol::RequestRealmsList => {
    		let realms: Vec<RealmId> = realms.iter().map(|realm| {
    			realm.id
    		}).collect();
			send_response(&RealmsProtocol::RealmsList(SelectionStorage::new_from(&realms)), &stream)?;
    		requests.push((client_id, request, Local::now()));
        },
        RealmsProtocol::RequestNewRealm => {
        	let id = realms.len();
	        let mut realm = client.realm_variant.create(id);
			send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
    		realms.push(realm);
    		requests.push((client_id, request, Local::now()));
        },
        RealmsProtocol::RequestRealm(realm_id) => {
        	if realms.len() > realm_id {
        	    if let Some(realm) = realms.get_mut(realm_id) {
					send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
        	    } else {
					send_response(&RealmsProtocol::Void, &stream)?;
        	    }
        	} else {
        		// send new realm on miss
	        	let id = realms.len();
	        	let realm = client.realm_variant.create(id);
				send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
	    		realms.push(realm);
        	}
    		requests.push((client_id, request, Local::now()));
        },
        RealmsProtocol::Explorer(Move::ChangeRegion(realm_id, region_id, explorer_id)) => {
        	if let Some(explorer) = realms.get_mut(realm_id).explorer(explorer_id) {
        	    explorer.region = Some(region_id);
        	}
        	if let Some(mut realm) = realms.get_mut(realm_id) {
    	    	client.realm_variant.state(realm);
				send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
        	} else {
				send_response(&RealmsProtocol::Void, &stream)?;
    	    }
    		requests.push((client_id, request, Local::now()));
        },
        RealmsProtocol::Explorer(Move::Action(realm_id, region_id, explorer_id, action)) => {
			let mut allowed = realms.get_mut(realm_id).region_explorer(region_id, explorer_id).is_some();
			if let Some(mut region) = realms.get_mut(realm_id).region(region_id) {
			    if allowed {
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
    				    		allowed = false;
    				    	}
    				    },
    				    ExplorerAction::Sail => {},
    				    ExplorerAction::Wait => {}
    				}
    	    	}
			}
			if let Some(mut realm) = realms.get_mut(realm_id) {
	        	if allowed {
    		    	client.realm_variant.state(realm);
					send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
	        	} else {
					send_response(&RealmsProtocol::Void, &stream)?;
	        	}
			} else {
				send_response(&RealmsProtocol::Void, &stream)?;
    	    }
        	if allowed {
    			requests.push((client_id, RealmsProtocol::Explorer(Move::Action(realm_id, region_id, explorer_id, action)), Local::now()));
        	}
        },
        RealmsProtocol::DropEquipment(realm_id, region_id, explorer_id, item) => {
        	if let Some(region) = realms.get_mut(realm_id).explorer_region(explorer_id) {
        	    region.particularities.insert(Particularity::Item(item.clone()));
        	}
        	if let Some(explorer) = realms.get_mut(realm_id).region_explorer(region_id, explorer_id) {
        	    let mut item_index_to_remove: Option<usize> = None;
				for (index, inventory_item) in explorer.inventory.iter_mut().enumerate() {
				    if let ExplorerItem::Equipment(equipment) = inventory_item {
				    	if *equipment == item {
				        	item_index_to_remove = Some(index);
				    	}
				    }
				}
				if let Some(index) = item_index_to_remove {
					explorer.inventory.at(index);
					explorer.inventory.extract_current();
				}
        	}

			if let Some(mut realm) = realms.get_mut(realm_id) {
				send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
				requests.push((client_id, RealmsProtocol::DropEquipment(realm_id, region_id, explorer_id, item.clone()), Local::now()));
		    }
        },
        RealmsProtocol::PickEquipment(realm_id, region_id, explorer_id, item) => {
        	if let Some(region) = realms.get_mut(realm_id).explorer_region(explorer_id) {
        	    let mut item_index_to_remove: Option<usize> = None;
				for (index, particularity_item) in region.particularities.iter_mut().enumerate() {
				    if let Particularity::Item(equipment) = particularity_item {
				    	if *equipment == item {
				        	item_index_to_remove = Some(index);
				    	}
				    }
				}
				if let Some(index) = item_index_to_remove {
					region.particularities.at(index);
					region.particularities.extract_current();
				}
        	}
        	if let Some(explorer) = realms.get_mut(realm_id).region_explorer(region_id, explorer_id) {
        	    explorer.inventory.insert(ExplorerItem::Equipment(item.clone()));
        	}

			if let Some(mut realm) = realms.get_mut(realm_id) {
				send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
				requests.push((client_id, RealmsProtocol::PickEquipment(realm_id, region_id, explorer_id, item.clone()), Local::now()));
		    }
        },
        RealmsProtocol::ForgetParticularity(realm_id, region_id, explorer_id, particularity) => {
		    if let Some(explorer) = realms.get_mut(realm_id).explorer(explorer_id) {
        	    let mut item_index_to_remove: Option<usize> = None;
				for (index, inventory_item) in explorer.inventory.iter_mut().enumerate() {
				    if let ExplorerItem::Particularity(investigated_region_id, investigated_item) = inventory_item {
				    	if particularity == *investigated_item && region_id == *investigated_region_id {
				        	item_index_to_remove = Some(index);
				    	}
				    }
				}
				if let Some(index) = item_index_to_remove {
					explorer.inventory.at(index);
					explorer.inventory.extract_current();
				}
        	}
        	
		    if let Some(mut realm) = realms.get_mut(realm_id) {
	        	send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
				requests.push((client_id, RealmsProtocol::ForgetParticularity(realm_id, region_id, explorer_id, particularity.clone()), Local::now()));
			} else {
				send_response(&RealmsProtocol::Void, &stream)?;
    	    }
        },
        RealmsProtocol::InvestigateParticularity(realm_id, region_id, explorer_id, item) => {
        	if let Some(explorer) = realms.get_mut(realm_id).region_explorer(region_id, explorer_id) {
        	    explorer.inventory.insert(ExplorerItem::Particularity(region_id, item.clone()));
        	}

		    if let Some(mut realm) = realms.get_mut(realm_id) {
	        	send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
				requests.push((client_id, RealmsProtocol::InvestigateParticularity(realm_id, region_id, explorer_id, item.clone()), Local::now()));
			} else {
				send_response(&RealmsProtocol::Void, &stream)?;
    	    }
        },
        RealmsProtocol::Quit => {
			send_response(&RealmsProtocol::Quit, &stream)?;
    		requests.push((client_id, request, Local::now()));
	    	client.connected = false;
			stream.shutdown(Shutdown::Both).expect("stream could not shut down.");
        },
        _ => {
			send_response(&RealmsProtocol::Void, &stream)?;
		}
    }

    Ok(())
}

fn send_response(data: &RealmsProtocol, mut stream: &TcpStream) -> Result<(), io::Error> {
	let raw = serialize(data).expect("could not serialize data for response.");
	stream.write(&raw).expect("could not write to tcp stream.");
	stream.flush().expect("could not flush response stream.");
	Ok(())
}
