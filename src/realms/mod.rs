
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
                tutorial::state(self);
            }
        }
    }  

    pub fn valid_move(&self, explorer: ExplorerId, region: RegionId) -> bool {
        match self.variant {
            RealmVariant::Tutorial => {
                tutorial::valid_move(self, explorer, region)
            }
        }
    }

    pub fn valid_action(&self, explorer: ExplorerId, region: RegionId, action: &ExplorerAction) -> bool {
        match self.variant {
            RealmVariant::Tutorial => {
                tutorial::valid_action(self, explorer, region, action)
            }
        }
    }      
}

