
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

			    match request {
			        RealmsProtocol::Register => {
			        	let id = Uuid::new_v4();
						send_response(&RealmsProtocol::Connect(id), &stream)?;
						self.clients.push(Client::new(id));
			    		self.requests.push((id, request, Local::now()));
			        },
			        RealmsProtocol::Connect(id) => {
			        	let mut client_found = false;
			        	for mut client in &mut self.clients {
			        	    if client.id == id {
			        	    	client.connected = true;
			        	    	client_found = true;
			        	    	client.time = Local::now();
			        	    }
			        	}

			        	if client_found {
							send_response(&RealmsProtocol::Connect(id), &stream)?;
			        	} else {
			        		let id = Uuid::new_v4();
							send_response(&RealmsProtocol::Connect(id), &stream)?;
							self.clients.push(Client::new(id));
			        	}
			    		self.requests.push((id, request, Local::now()));
			        },
			        RealmsProtocol::RequestRealmsList => {
		        		let realms: Vec<RealmId> = self.realms.iter().map(|realm| {
		        			realm.id
		        		}).collect();
						send_response(&RealmsProtocol::RealmsList(SelectionStorage::new_from(&realms)), &stream)?;
			    		self.requests.push((client_id, request, Local::now()));
			    		for client in &mut self.clients {
			    		    if client.id == client_id {
			    		    	client.time = Local::now();
			    		    }
			    		}
			        },
			        RealmsProtocol::RequestNewRealm => {
			        	let id = self.realms.len();
				        let mut realm = Realm::new(id);
			        	for client in &mut self.clients {
			    		    if client.id == client_id {
			    		    	realm = client.realm_variant.create(id)
			    		    }
			    		}
						send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
			    		self.realms.push(realm);
			    		self.requests.push((client_id, request, Local::now()));
			    		for client in &mut self.clients {
			    		    if client.id == client_id {
			    		    	client.time = Local::now();
			    		    }
			    		}
			        },
			        RealmsProtocol::RequestRealm(realm_id) => {
			        	let mut realm;
			        	if self.realms.len() > realm_id {
			        	    realm = self.realms[realm_id].clone();
							send_response(&RealmsProtocol::Realm(realm), &stream)?;
			        	} else {
			        		// send new realm on miss
				        	let id = self.realms.len();
				        	realm = Realm::new(id);
				        	for client in &mut self.clients {
				    		    if client.id == client_id {
				    		    	realm = client.realm_variant.create(id)
				    		    }
				    		}
							send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
				    		self.realms.push(realm);
			        	}
			    		self.requests.push((client_id, request, Local::now()));
			    		for client in &mut self.clients {
			    		    if client.id == client_id {
			    		    	client.time = Local::now();
			    		    }
			    		}
			        },
			        RealmsProtocol::Explorer(Move::ChangeRegion(realm_id, region_id, explorer_id)) => {
			        	for mut realm in &mut self.realms {
			        	    if realm_id == realm.id {
			        	    	for explorer in &mut realm.expedition.explorers.iter_mut() {
			        	    		if explorer.id == explorer_id {
			        	    			explorer.region = Some(region_id);
			        	    		}

			        	    	}
			        	    	for client in &mut self.clients {
					    		    if client.id == client_id {
					    		    	client.realm_variant.state(&mut realm);
					    		    	client.time = Local::now();
					    		    }
					    		}
								send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
			        	    }
			        	}
			    		self.requests.push((client_id, request, Local::now()));
			        },
			        RealmsProtocol::Explorer(Move::Action(realm_id, region_id, explorer_id, action)) => {
    	    			let mut allowed = false;
			        	for mut realm in &mut self.realms {
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
			        	    		for client in &mut self.clients {
						    		    if client.id == client_id {
						    		    	client.realm_variant.state(&mut realm);
						    		    	client.time = Local::now();
						    		    }
						    		}
									send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
					        	} else {
									send_response(&RealmsProtocol::Void, &stream)?;
					        	}
			        	    }
			        	}
			        	if allowed {
			    			self.requests.push((client_id, RealmsProtocol::Explorer(Move::Action(realm_id, region_id, explorer_id, action)), Local::now()));
			        	}
			        },
			        RealmsProtocol::DropEquipment(realm_id, region_id, explorer_id, item) => {
						for realm in &mut self.realms {
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
		        	    		for client in &mut self.clients {
					    		    if client.id == client_id {
					    		    	client.time = Local::now();
					    		    }
					    		}
								send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
		    					self.requests.push((client_id, RealmsProtocol::DropEquipment(realm_id, region_id, explorer_id, item.clone()), Local::now()));
					        }
					    }
			        },
			        RealmsProtocol::PickEquipment(realm_id, region_id, explorer_id, item) => {
						for realm in &mut self.realms {
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
		        	    		for client in &mut self.clients {
					    		    if client.id == client_id {
					    		    	client.time = Local::now();
					    		    }
					    		}
								send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
		    					self.requests.push((client_id, RealmsProtocol::PickEquipment(realm_id, region_id, explorer_id, item.clone()), Local::now()));
					        }
					    }
			        },
			        RealmsProtocol::ForgetParticularity(realm_id, region_id, explorer_id, particularity) => {
						for realm in &mut self.realms {
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
		        	    		for client in &mut self.clients {
					    		    if client.id == client_id {
					    		    	client.time = Local::now();
					    		    }
					    		}
								send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
		    					self.requests.push((client_id, RealmsProtocol::ForgetParticularity(realm_id, region_id, explorer_id, particularity.clone()), Local::now()));
					        }
					    }
			        },
			        RealmsProtocol::InvestigateParticularity(realm_id, region_id, explorer_id, item) => {
						for realm in &mut self.realms {
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
		        	    		for client in &mut self.clients {
					    		    if client.id == client_id {
					    		    	client.time = Local::now();
					    		    }
					    		}
								send_response(&RealmsProtocol::Realm(realm.clone()), &stream)?;
		    					self.requests.push((client_id, RealmsProtocol::InvestigateParticularity(realm_id, region_id, explorer_id, item.clone()), Local::now()));
					        }
					    }
			        },
			        RealmsProtocol::Quit => {
						send_response(&RealmsProtocol::Quit, &stream)?;
						stream.shutdown(Shutdown::Both).expect("stream could not shut down.");
			    		self.requests.push((client_id, request, Local::now()));
			    		for client in &mut self.clients {
			    		    if client.id == client_id {
			    		    	client.connected = false;
			    		    	client.time = Local::now();
			    		    }
			    		}
			    		// draw dashboard update before client exits
			    		draw(t, &self.requests, &self.clients, &self.realms)?;
			        	break;
			        },
			        _ => {
						send_response(&RealmsProtocol::Void, &stream)?;
					}
			    }
			    draw(t, &self.requests, &self.clients, &self.realms)?;
			}
	    }
	    t.show_cursor().unwrap();
	    t.clear().unwrap();

	    Ok(())
	}
}

fn send_response(data: &RealmsProtocol, mut stream: &TcpStream) -> Result<(), io::Error> {
	let raw = serialize(data).expect("could not serialize data for response.");
	stream.write(&raw).expect("could not write to tcp stream.");
	stream.flush().expect("could not flush response stream.");
	Ok(())
}


