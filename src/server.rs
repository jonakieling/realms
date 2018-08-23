
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
	pub clients: Vec<Client>
}

#[derive(Clone)]
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
			    let mut current_client: Option<Client> = None;
			    {
			    	for client in &self.clients {
		        	    if client.id == client_id {
		        	    	current_client = Some(client.clone());
		        	    }
		        	}
			    }

			    // seperate client and no-client request handling
	        	if let Some(ref mut client) = current_client {
	        	    handle_request(&mut self.requests, &mut self.realms, &mut stream, client, request)?;

	        	    if !client.connected {
	        	    	// draw dashboard before client exits and then break the stream loop
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

fn handle_connecting_requests(clients: &mut Vec<Client>, requests: &mut Vec<(ClientId, RealmsProtocol, DateTime<Local>)>, stream: &mut TcpStream, request: RealmsProtocol) -> Result<(), io::Error> {
	match request {
		// a connect request when the server could not find the id matching client acts as a register request.
        RealmsProtocol::Register | RealmsProtocol::Connect(_) => {
        	let id = Uuid::new_v4();
			send_response(&RealmsProtocol::Connect(id), &stream)?;
			clients.push(Client::new(id));
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
        	let mut realm;
        	if realms.len() > realm_id {
        	    realm = realms[realm_id].clone();
				send_response(&RealmsProtocol::Realm(realm), &stream)?;
        	} else {
        		// send new realm on miss
	        	let id = realms.len();
	        	realm = client.realm_variant.create(id);
				send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
	    		realms.push(realm);
        	}
    		requests.push((client_id, request, Local::now()));
        },
        RealmsProtocol::Explorer(Move::ChangeRegion(realm_id, region_id, explorer_id)) => {
        	for mut realm in realms {
        	    if realm_id == realm.id {
        	    	for explorer in &mut realm.expedition.explorers.iter_mut() {
        	    		if explorer.id == explorer_id {
        	    			explorer.region = Some(region_id);
        	    		}

        	    	}
        	    	client.realm_variant.state(&mut realm);
					send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
        	    }
        	}
    		requests.push((client_id, request, Local::now()));
        },
        RealmsProtocol::Explorer(Move::Action(realm_id, region_id, explorer_id, action)) => {
			let mut allowed = false;
        	for mut realm in realms {
        	    if realm_id == realm.id {
    	    		for region in &mut realm.island.regions.iter_mut() {
        	    		if region_id == region.id {
        	    			for explorer in &mut realm.expedition.explorers.iter() {
		        	    		if explorer.id == explorer_id {
		        	    			if let Some(explorer_region) = explorer.region {
		        	    				if explorer_region == region_id {
		        	    					allowed = true;
		        	    				}
		        	    			}
		        	    		}

		        	    	}
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
    	    		}
		        	if allowed {
	    		    	client.realm_variant.state(&mut realm);
						send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
		        	} else {
						send_response(&RealmsProtocol::Void, &stream)?;
		        	}
        	    }
        	}
        	if allowed {
    			requests.push((client_id, RealmsProtocol::Explorer(Move::Action(realm_id, region_id, explorer_id, action)), Local::now()));
        	}
        },
        RealmsProtocol::DropEquipment(realm_id, region_id, explorer_id, item) => {
			for realm in realms {
        	    if realm_id == realm.id {
    	    		for region in &mut realm.island.regions.iter_mut() {
        	    		if region_id == region.id {
        	    			for explorer in realm.expedition.explorers.iter_mut() {
		        	    		if explorer.id == explorer_id {
		        	    			if let Some(explorer_region) = explorer.region {
		        	    				if explorer_region == region_id {
		        	    					region.particularities.insert(Particularity::Item(item.clone()));
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
		        	    			}
		        	    		}
		        	    	}
		        	    }
		        	}
					send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
					requests.push((client_id, RealmsProtocol::DropEquipment(realm_id, region_id, explorer_id, item.clone()), Local::now()));
		        }
		    }
        },
        RealmsProtocol::PickEquipment(realm_id, region_id, explorer_id, item) => {
			for realm in realms {
        	    if realm_id == realm.id {
    	    		for region in &mut realm.island.regions.iter_mut() {
        	    		if region_id == region.id {
        	    			for explorer in realm.expedition.explorers.iter_mut() {
		        	    		if explorer.id == explorer_id {
		        	    			if let Some(explorer_region) = explorer.region {
		        	    				if explorer_region == region_id {
		        	    					explorer.inventory.insert(ExplorerItem::Equipment(item.clone()));
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
		        	    			}
		        	    		}
		        	    	}
		        	    }
		        	}
					send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
					requests.push((client_id, RealmsProtocol::PickEquipment(realm_id, region_id, explorer_id, item.clone()), Local::now()));
		        }
		    }
        },
        RealmsProtocol::ForgetParticularity(realm_id, region_id, explorer_id, particularity) => {
			for realm in realms {
        	    if realm_id == realm.id {
    	    		for explorer in realm.expedition.explorers.iter_mut() {
        	    		if explorer.id == explorer_id {
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
        	    	}
					send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
					requests.push((client_id, RealmsProtocol::ForgetParticularity(realm_id, region_id, explorer_id, particularity.clone()), Local::now()));
		        }
		    }
        },
        RealmsProtocol::InvestigateParticularity(realm_id, region_id, explorer_id, item) => {
			for realm in realms {
        	    if realm_id == realm.id {
    	    		for region in &mut realm.island.regions.iter_mut() {
        	    		if region_id == region.id {
        	    			for explorer in realm.expedition.explorers.iter_mut() {
		        	    		if explorer.id == explorer_id {
		        	    			if let Some(explorer_region) = explorer.region {
		        	    				if explorer_region == region_id {
		        	    					explorer.inventory.insert(ExplorerItem::Particularity(region_id, item.clone()));
		        	    				}
		        	    			}
		        	    		}
		        	    	}
		        	    }
		        	}
					send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
					requests.push((client_id, RealmsProtocol::InvestigateParticularity(realm_id, region_id, explorer_id, item.clone()), Local::now()));
		        }
		    }
        },
        RealmsProtocol::Quit => {
			send_response(&RealmsProtocol::Quit, &stream)?;
			stream.shutdown(Shutdown::Both).expect("stream could not shut down.");
    		requests.push((client_id, request, Local::now()));
	    	client.connected = false;
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
