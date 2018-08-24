
use rand::{thread_rng, distributions::Uniform, Rng};

use tokens::*;
use tokens::Equipment::*;
use utility::*;
use itertools::Itertools;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RealmVariant {
	Tutorial(RealmTemplate),
	// PrologueTheQueen(RealmTemplate)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RealmTemplate {
    regions: Vec<Region>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RealmTemplateVariant {
    Tutorial,
    // PrologueTheQueen
}

impl RealmTemplate {
    pub fn new(variant: RealmTemplateVariant) -> RealmTemplate {
        match variant {
            RealmTemplateVariant::Tutorial => RealmTemplate {
                regions: tutorial_regions()
            }
        }
        
    }
}

pub trait RealmStrategy {
    fn create(&self, id: usize) -> Realm;
    fn state(&self, realm: &mut Realm);
    fn valid_move(&self, realm: &Realm, explorer: ExplorerId, region: RegionId) -> bool;
    fn valid_action(&self, realm: &Realm, explorer: ExplorerId, region: RegionId, action: &ExplorerAction) -> bool;
}

impl RealmStrategy for RealmVariant {
    fn create(&self, id: usize) -> Realm {
        match self {
            RealmVariant::Tutorial(template) => { realm_tutorial(id, template) }
        }
    }

    fn state(&self, realm: &mut Realm) {
        match self {
            RealmVariant::Tutorial(_template) => {
                let mut embarked = 0;
                for explorer in realm.expedition.explorers.iter() {
                    if explorer.region.is_some() {
                        embarked += 1;
                    }
                }
                if embarked == realm.expedition.explorers.iter().len() {
                    realm.story = "all explorers have embarked. you can keep playing around.".to_string();
                    realm.done = true;
                }
                realm.age += 1;
            }
        }
    }  

    fn valid_move(&self, _realm: &Realm, _explorer: ExplorerId, _region: RegionId) -> bool {
        match self {
            RealmVariant::Tutorial(_template) => {
                // no movement restrictions for tutorial
                true
            }
        }
    }

    fn valid_action(&self, _realm: &Realm, _explorer: ExplorerId, _region: RegionId, _action: &ExplorerAction) -> bool {
        match self {
            RealmVariant::Tutorial(_template) => {
                // no restrictions on actions for tutorial
                true
            }
        }
    }      
}

fn realm_tutorial(id: usize, template: &RealmTemplate) -> Realm {
    let mut rng = thread_rng();

    // todo: here the initial regions of the realm should be assembled
    let regions = template.regions.clone();

    let island = Island {
        regions: SelectionStorage::new_from(&regions)
    };

    // Pots
    // Tinder
    // Firewood(usize)
    // Coal(usize)
    // Gold(usize)
    // Coins(usize)
    // Tools
    // Flint
    // Wax
    // SealStamp
    // Blankets
    // Herbs(usize)
    // Food(usize)
    // Pipe
    // Telescope
    // Compass
    // Parchment(usize)
    // Map
    // Knife
    // Spear
    // Bow
    // Arrows(usize)
    // Canoe
    // Raft
    let mut explorers = SelectionStorage::new();
    let how_many_explorers = rng.sample(&Uniform::new_inclusive(3, 5));

    explorers.insert(Explorer {
        id: 0,
        traits: SelectionStorage::new_from(&vec![ExplorerTrait::Ranger]),
        region: None,
        inventory: SelectionStorage::new_from(&vec![
            ExplorerItem::Equipment(Bow),
            ExplorerItem::Equipment(Arrows(75)),
            ExplorerItem::Equipment(Knife),
            ExplorerItem::Equipment(Coins(110)),
            ExplorerItem::Equipment(Telescope),
            ExplorerItem::Equipment(Herbs(20))])
    });
    explorers.insert(Explorer {
        id: 1,
        traits: SelectionStorage::new_from(&vec![ExplorerTrait::Builder]),
        region: None,
        inventory: SelectionStorage::new_from(&vec![
            ExplorerItem::Equipment(Tools),
            ExplorerItem::Equipment(Food(10)),
            ExplorerItem::Equipment(Pipe),
            ExplorerItem::Equipment(Blankets),
            ExplorerItem::Equipment(Knife)])
    });
    explorers.insert(Explorer {
        id: 2,
        traits: SelectionStorage::new_from(&vec![]),
        region: None,
        inventory: SelectionStorage::new_from(&vec![
            ExplorerItem::Equipment(Pots),
            ExplorerItem::Equipment(Tinder),
            ExplorerItem::Equipment(Firewood(4)),
            ExplorerItem::Equipment(Flint),
            ExplorerItem::Equipment(Rope)])
    });

    if how_many_explorers > 3 {
        explorers.insert(Explorer {
            id: 3,
            traits: SelectionStorage::new_from(&vec![ExplorerTrait::Cartographer]),
            region: None,
            inventory: SelectionStorage::new_from(&vec![
                ExplorerItem::Equipment(Parchment(10)),
                ExplorerItem::Equipment(Map),
                ExplorerItem::Equipment(Rope),
                ExplorerItem::Equipment(Wax),
                ExplorerItem::Equipment(SealStamp)])
        });
    }

    if how_many_explorers > 4 {
        let mut explorer = Explorer {
            id: 4,
            traits: SelectionStorage::new_from(&vec![ExplorerTrait::Sailor]),
            region: None,
            inventory: SelectionStorage::new_from(&vec![
                ExplorerItem::Equipment(Coins(32)),
                ExplorerItem::Equipment(Gold(4)),
                ExplorerItem::Equipment(Rope),
                ExplorerItem::Equipment(Knife),
                ExplorerItem::Equipment(Compass),
                ExplorerItem::Equipment(Telescope)])
        };
        let canoe_or_not = rng.sample(&Uniform::new_inclusive(0, 1));
        if canoe_or_not == 1 {
            explorer.inventory.insert(ExplorerItem::Equipment(Canoe));
        }
        explorers.insert(explorer);
    }

    let expedition = Expedition {
        explorers
    };

    Realm {
        id,
        island,
        expedition,
        age: 0,
        title: "tutorial".to_string(),
        story: "embark all explorers.".to_string(),
        objectives: vec![RealmObjective::EmbarkExplorers],
        completed: vec![],
        done: false
    }
}

fn tutorial_regions() -> Vec<Region> {
    let mut rng = thread_rng();
    let mut rng2 = thread_rng();
    let mut region_id = 0;

    rng.sample_iter(&Uniform::new_inclusive(1, 4)).take(19).map(|number| {
        let terrain = match number {
            1 => Terrain::Coast,
            2 => Terrain::Planes,
            3 => Terrain::Forest,
            _ => Terrain::Mountain,
        };

        // Town
        // River
        // Carravan
        // Merchant
        // Camp
        // Gear(Gear)
        // Canyon
        // Bolders
        // Grasland
        // Creek
        // Grove
        // Cliffs
        // Island
        // Lake
        // Pond
        // Clearing
        let particularities: Vec<Particularity> = match terrain {
            Terrain::Coast => {
                let how_many_particularities = rng2.sample(&Uniform::new_inclusive(1, 2));

                rng2.sample_iter(&Uniform::new_inclusive(1, 9)).take(how_many_particularities).map(|number| {
                    match number {
                        1 => Particularity::Town,
                        2 => Particularity::River,
                        3 => Particularity::Cliffs,
                        4 => Particularity::Cliffs,
                        5 => Particularity::Cliffs,
                        6 => Particularity::Island,
                        7 => Particularity::Island,
                        8 => Particularity::Ship,
                        _ => Particularity::Carravan
                    }
                }).unique().collect()
            },
            Terrain::Planes => {
                let how_many_particularities = rng2.sample(&Uniform::new_inclusive(1, 3));

                rng2.sample_iter(&Uniform::new_inclusive(1, 10)).take(how_many_particularities).map(|number| {
                    match number {
                        1 => Particularity::Town,
                        2 => Particularity::Merchant,
                        3 => Particularity::Grove,
                        4 => Particularity::Grove,
                        5 => Particularity::Creek,
                        6 => Particularity::Grasland,
                        7 => Particularity::Grasland,
                        8 => Particularity::Grasland,
                        9 => Particularity::River,
                        _ => Particularity::Carravan
                    }
                }).unique().collect()
            },
            Terrain::Forest => {
                let how_many_particularities = rng2.sample(&Uniform::new_inclusive(0, 2));

                rng2.sample_iter(&Uniform::new_inclusive(1, 9)).take(how_many_particularities).map(|number| {
                    match number {
                        1 => Particularity::Town,
                        2 => Particularity::River,
                        3 => Particularity::Creek,
                        4 => Particularity::Creek,
                        5 => Particularity::Clearing,
                        6 => Particularity::Clearing,
                        7 => Particularity::Clearing,
                        8 => Particularity::Pond,
                        _ => Particularity::Carravan
                    }
                }).unique().collect()
            },
            Terrain::Mountain => {
                let how_many_particularities = rng2.sample(&Uniform::new_inclusive(0, 1));

                rng2.sample_iter(&Uniform::new_inclusive(1, 8)).take(how_many_particularities).map(|number| {
                    match number {
                        1 => Particularity::Town,
                        2 => Particularity::River,
                        3 => Particularity::Canyon,
                        4 => Particularity::Bolders,
                        5 => Particularity::Bolders,
                        6 => Particularity::Bolders,
                        7 => Particularity::Lake,
                        _ => Particularity::Carravan
                    }
                }).unique().collect()
            }
        };

        let resources = match terrain {
            Terrain::Planes => 6,
            Terrain::Forest => 5,
            Terrain::Coast => 3,
            Terrain::Mountain => 2,
        };

        let region = Region {
            id: region_id,
            terrain,
            particularities: SelectionStorage::new_from(&particularities),
            buildings: SelectionStorage::new(),
            mapped: false,
            resources
        };
        region_id += 1;

        region
    }).collect()
}