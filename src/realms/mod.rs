
use tokens::*;
use utility::*;

mod tutorial;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RealmVariant {
	Tutorial,
	// PrologueTheQueen(RealmTemplate)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RealmTemplate {
    pub regions: SelectionHashMap<Region>,
    pub explorers: Vec<Explorer>
}

pub struct RealmStrategy {
    pub variant: RealmVariant,
    pub realm: Realm,
    pub template: RealmTemplate
}

impl RealmStrategy {
    pub fn new(id: usize, variant: RealmVariant) -> RealmStrategy {
        match variant {
            RealmVariant::Tutorial => {
                tutorial::new(id)
            }
        }
    }

    pub fn state(&mut self) {
        match self.variant {
            RealmVariant::Tutorial => {

                for (_, region) in self.realm.island.regions.iter_mut() {
                    region.resources = 0;
                    region.buildings = SelectionStorage::new();
                    region.particularities = SelectionStorage::new();
                    region.sight = RegionVisibility::None;
                }
                
                for (_, region) in self.template.regions.iter() {
                    if region.mapped {
                        let mut region = region.clone();
                        region.sight = RegionVisibility::Partial;
                        self.realm.island.regions.insert(region.id, region.clone());
                    }
                }

                let mut embarked = 0;
                for explorer in self.realm.expedition.explorers.iter() {
                    if let Some(explorer_region) = explorer.region {
                        if let Some(explorer_region) = self.template.regions.storage().get(&explorer_region) {
                            
                            for neighbor in &explorer_region.neighbors {
                                if let Some(region) = self.template.regions.storage().get(&neighbor).clone() {
                                    let mut region = region.clone();
                                    region.sight = RegionVisibility::Partial;
                                    self.realm.island.regions.insert(region.id, region);
                                }
                            }

                            let mut region = explorer_region.clone();
                            region.sight = RegionVisibility::Live;
                            self.realm.island.regions.insert(region.id, region);
                        }
                    }
                    if explorer.region.is_some() {
                        embarked += 1;
                    }
                }

                if embarked == self.realm.expedition.explorers.iter().len() {
                    self.realm.completed.push(RealmObjective::EmbarkExplorers);
                    self.realm.story = "all explorers have embarked. you can keep playing around.".to_string();
                    self.realm.done = true;
                }
                self.realm.age += 1;
            }
        }
    }  

    pub fn valid_move(&self, _explorer: ExplorerId, _region: RegionId) -> bool {
        match self.variant {
            RealmVariant::Tutorial => {
                // no movement restrictions for tutorial
                true
            }
        }
    }

    pub fn valid_action(&self, _explorer: ExplorerId, _region: RegionId, _action: &ExplorerAction) -> bool {
        match self.variant {
            RealmVariant::Tutorial => {
                // no restrictions on actions for tutorial
                true
            }
        }
    }      
}

