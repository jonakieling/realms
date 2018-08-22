
use rand::{thread_rng, distributions::Uniform, Rng};

use tokens::*;
use tokens::Equipment::*;
use utility::*;
use itertools::Itertools;

pub enum Realms {
	Dev,
	// PrologueTheQueen
}

impl Realms {
    pub fn create(self, id: usize) -> Realm {
    	match self {
    		Realms::Dev => { realm_dev(id) }
    	}
    }
}

fn realm_dev(id: usize) -> Realm {
	let mut rng = thread_rng();
    let mut rng2 = thread_rng();
    let mut region_id = 0;
    let regions: Vec<Region> = rng.sample_iter(&Uniform::new_inclusive(1, 4)).take(19).map(|number| {
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
                        8 => Particularity::Item(Equipment::Boat),
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

        let region = Region {
            id: region_id,
            terrain,
            particularities: SelectionStorage::new_from(&particularities),
            buildings: SelectionStorage::new(),
            mapped: false,
            resources: 10
        };
        region_id += 1;

        region
    }).collect();

    let island = Island {
        regions: SelectionStorage::new_from(&regions)
    };

    let expedition = Expedition {
        explorers: SelectionStorage::new_from(&vec![
            Explorer { id: 0, traits: SelectionStorage::new_from(&vec![ExplorerTrait::Ranger]), region: None, inventory: SelectionStorage::new_from(&vec![Item::Equipment(SurvivalKit)]) },
            Explorer { id: 1, traits: SelectionStorage::new_from(&vec![ExplorerTrait::Cartographer]), region: None, inventory: SelectionStorage::new_from(&vec![Item::Equipment(ClimbingGear)]) },
            Explorer { id: 2, traits: SelectionStorage::new_from(&vec![ExplorerTrait::Engineer]), region: None, inventory: SelectionStorage::new_from(&vec![Item::Equipment(HotAirBalloon)]) },
            Explorer { id: 3, traits: SelectionStorage::new_from(&vec![ExplorerTrait::Sailor]), region: None, inventory: SelectionStorage::new_from(&vec![Item::Equipment(Boat)]) }
        ])
    };

    Realm {
    	id,
    	island,
    	expedition,
    	age: 0
    }
}