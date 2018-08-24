
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
	pub realms_list: SelectionStorage<RealmId>,
	pub realm_variant: RealmVariant,
	pub completed_variants: Vec<RealmVariant>
}

impl Client {
	pub fn new(id: ClientId) -> Client {
		Client {
			id,
			connected: true,
			time: Local::now(),
			realms_list: SelectionStorage::new(),
			realm_variant: RealmVariant::Tutorial(RealmTemplate::new(RealmTemplateVariant::Tutorial)),
			completed_variants: vec![]
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
	        		let response = handle_request(&mut self.requests, &mut self.realms, &mut client, request);
	        	    send_response(&response, &stream)?;

	        		let disconnect = !client.connected;

	        		// 'update' client in list
	        		self.clients.insert(client.id, client);

	        	    if disconnect {
	        	    	// draw update before client exits
			    		draw(t, &self.requests, &self.clients, &self.realms)?;
						stream.shutdown(Shutdown::Both).expect("stream could not shut down.");
	        	    	break;
	        	    }
	        	} else {
	        	    let response = handle_connecting_requests(&mut self.clients, &mut self.requests, request);
	        	    send_response(&response, &stream)?;
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

fn handle_connecting_requests(clients: &mut HashMap<Uuid, Client>, requests: &mut Vec<(ClientId, RealmsProtocol, DateTime<Local>)>, request: RealmsProtocol) -> RealmsProtocol {
	match request {
		// a connect request when the server could not find the id matching client acts as a register request.
        RealmsProtocol::Register | RealmsProtocol::Connect(_) => {
        	let id = Uuid::new_v4();
			clients.insert(id, Client::new(id));
    		requests.push((id, request, Local::now()));

    		RealmsProtocol::Connect(id)
        },
        _ => {
			RealmsProtocol::Void
		}
	}
}

fn handle_request(requests: &mut Vec<(ClientId, RealmsProtocol, DateTime<Local>)>, realms: &mut Vec<Realm>, client: &mut Client, request: RealmsProtocol) -> RealmsProtocol {
	let client_id = client.id;
	client.time = Local::now();

	match request {
		RealmsProtocol::Connect(id) => {
			client.connected = true;
    		requests.push((id, request, Local::now()));

    		RealmsProtocol::Connect(id)
        },
        RealmsProtocol::RequestRealmsList => {
    		requests.push((client_id, request, Local::now()));

    		RealmsProtocol::RealmsList(client.realms_list.clone())
        },
        RealmsProtocol::RequestNewRealm => {
        	let id = realms.len();
	        let mut realm = client.realm_variant.create(id);
    		realms.push(realm.clone());
    		client.realms_list.insert(realm.id);
    		requests.push((client_id, request, Local::now()));

    		RealmsProtocol::Realm(realm)
        },
        RealmsProtocol::RequestRealm(realm_id) => {
        	if realms.len() > realm_id {
        	    if let Some(realm) = realms.get_mut(realm_id) {
    				requests.push((client_id, request, Local::now()));
					RealmsProtocol::Realm(realm.clone())
        	    } else {
					RealmsProtocol::Void
        	    }
        	} else {
        		// send new realm on miss
	        	let id = realms.len();
	        	let realm = client.realm_variant.create(id);
				realms.push(realm.clone());
    			client.realms_list.insert(realm.id);
    			requests.push((client_id, request, Local::now()));

				RealmsProtocol::Realm(realm)
        	}
        },
        RealmsProtocol::Explorer(Move::ChangeRegion(realm_id, region_id, explorer_id)) => {
        	// todo: check consequences of move for realm template

        	let mut valid_move = false;
        	if let Some(realm) = realms.get(realm_id) {
        	    valid_move = client.realm_variant.valid_move(realm, explorer_id, region_id);
        	}

        	if valid_move {
	        	if let Some(explorer) = realms.get_mut(realm_id).explorer(explorer_id) {
	    	    	explorer.region = Some(region_id);
	        	}

	        	if let Some(mut realm) = realms.get_mut(realm_id) {
	        		let done_before = realm.done;
	    	    	client.realm_variant.state(realm);
	    	    	if realm.done && !done_before {
	    	    		client.completed_variants.push(client.realm_variant.clone());
	    	    	}
	    			requests.push((client_id, request, Local::now()));
					RealmsProtocol::Realm(realm.clone())
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
        	if let Some(realm) = realms.get(realm_id) {
        	    valid_action = client.realm_variant.valid_action(realm, explorer_id, region_id, &action);
        	}

			valid_action &= realms.get_mut(realm_id).region_explorer(region_id, explorer_id).is_some();
			if let Some(mut region) = realms.get_mut(realm_id).region(region_id) {
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
    	    	}
			}
			if let Some(mut realm) = realms.get_mut(realm_id) {
	        	if valid_action {
	        		let done_before = realm.done;
	    	    	client.realm_variant.state(realm);
	    	    	if realm.done && !done_before {
	    	    		client.completed_variants.push(client.realm_variant.clone());
	    	    	}
    		    	requests.push((client_id, RealmsProtocol::Explorer(Move::Action(realm_id, region_id, explorer_id, action)), Local::now()));
					RealmsProtocol::Realm(realm.clone())

	        	} else {
					RealmsProtocol::Void
	        	}
			} else {
				RealmsProtocol::Void
    	    }
        },
        RealmsProtocol::DropEquipment(realm_id, region_id, explorer_id, item) => {
        	// todo: drop item in realm template aswell

        	if let Some(region) = realms.get_mut(realm_id).explorer_region(explorer_id) {
        	    region.particularities.insert(Particularity::Item(item));
        	}
        	if let Some(explorer) = realms.get_mut(realm_id).explorer(explorer_id) {
        	    explorer.inventory.storage_mut().iter().position(move |ref n| **n == ExplorerItem::Equipment(item)).map(|equipment| {
    				explorer.inventory.storage_mut().remove(equipment);
        		});
        	}

			if let Some(mut realm) = realms.get_mut(realm_id) {
				requests.push((client_id, RealmsProtocol::DropEquipment(realm_id, region_id, explorer_id, item), Local::now()));

				RealmsProtocol::Realm(realm.clone())
		    } else {
		    	RealmsProtocol::Void
		    }
        },
        RealmsProtocol::PickEquipment(realm_id, region_id, explorer_id, item) => {
        	// todo: pick item in realm template aswell

        	if let Some(region) = realms.get_mut(realm_id).explorer_region(explorer_id) {
        		region.particularities.storage_mut().iter().position(move |ref n| **n == Particularity::Item(item)).map(|equipment| {
    				region.particularities.storage_mut().remove(equipment);
        		});
        	}
        	if let Some(explorer) = realms.get_mut(realm_id).region_explorer(region_id, explorer_id) {
        	    explorer.inventory.insert(ExplorerItem::Equipment(item));
        	}

			if let Some(mut realm) = realms.get_mut(realm_id) {
				requests.push((client_id, RealmsProtocol::PickEquipment(realm_id, region_id, explorer_id, item), Local::now()));

				RealmsProtocol::Realm(realm.clone())
		    } else {
		    	RealmsProtocol::Void
		    }
        },
        RealmsProtocol::ForgetParticularity(realm_id, region_id, explorer_id, particularity) => {
		    if let Some(explorer) = realms.get_mut(realm_id).explorer(explorer_id) {
        	    explorer.inventory.storage_mut().iter().position(move |ref n| **n == ExplorerItem::Particularity(region_id, particularity)).map(|equipment| {
    				explorer.inventory.storage_mut().remove(equipment);
        		});
        	}
        	
		    if let Some(mut realm) = realms.get_mut(realm_id) {
				requests.push((client_id, RealmsProtocol::ForgetParticularity(realm_id, region_id, explorer_id, particularity), Local::now()));

				RealmsProtocol::Realm(realm.clone())
			} else {
				RealmsProtocol::Void
    	    }
        },
        RealmsProtocol::InvestigateParticularity(realm_id, region_id, explorer_id, item) => {
        	if let Some(explorer) = realms.get_mut(realm_id).region_explorer(region_id, explorer_id) {
        	    explorer.inventory.insert(ExplorerItem::Particularity(region_id, item));
        	}

		    if let Some(mut realm) = realms.get_mut(realm_id) {
				requests.push((client_id, RealmsProtocol::InvestigateParticularity(realm_id, region_id, explorer_id, item), Local::now()));

				RealmsProtocol::Realm(realm.clone())
			} else {
				RealmsProtocol::Void
    	    }
        },
        RealmsProtocol::Quit => {
			requests.push((client_id, request, Local::now()));
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
