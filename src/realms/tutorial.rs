
use realms::RealmTemplate;
use realms::RealmStrategy;
use realms::RealmVariant;
use rand::{thread_rng, distributions::Uniform, Rng};

use tokens::*;
use tokens::Equipment::*;
use utility::*;
use itertools::Itertools;

use hex::*;

pub fn new(id: RealmId) -> RealmStrategy {
	let template = template();
    RealmStrategy { variant: RealmVariant::Tutorial, realm: realm(id, &template), template }
}

fn template() -> RealmTemplate {
	RealmTemplate {
        regions: regions(),
        explorers: explorers()
    }
}

fn realm(id: usize, template: &RealmTemplate) -> Realm {
    let mut regions = SelectionHashMap::new();
    for (id, region) in template.regions.iter().take(2) {
        let mut region = region.clone();
        region.sight = RegionVisibility::Complete;
        regions.insert(*id, region);
    }
    let island = Island {
        regions
    };

    let expedition = Expedition {
        explorers: SelectionStorage::new_from(&template.explorers)
    };

    Realm {
        id,
        island,
        expedition,
        age: 0,
        title: "tutorial".to_string(),
        story: "".to_string(),
        objectives: vec![RealmObjective::EmbarkExplorers],
        completed: vec![],
        done: false
    }
}

fn regions() -> SelectionHashMap<Region> {
    let mut rng = thread_rng();
    let mut rng2 = thread_rng();
    let mut region_id = 0;

    let mut regions = SelectionHashMap::new();

    let cols = 5;
    let rows = 5;
    let hexes = hexes(cols, rows);

    for number in rng.sample_iter(&Uniform::new_inclusive(1, 4)).take(cols * rows) {
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
            resources,
            sight: RegionVisibility::None,
            neighbors: vec![],
            hex_offset_coords: (0, 0)
        };
        region_id += 1;

        regions.insert(region.id, region);

        // here id matches index so we zip the hex neighbors onto the regions
        for ((_, mut region), hex) in regions.iter_mut().zip(hexes.iter()) {
            region.neighbors = hex.neighbors.clone();
            region.hex_offset_coords = hex.offset;
        }
    }

    regions
}

fn explorers() -> Vec<Explorer> {
    let mut rng = thread_rng();

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
    let mut explorers = vec![];
    let how_many_explorers = rng.sample(&Uniform::new_inclusive(3, 5));

    explorers.push(Explorer {
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
    explorers.push(Explorer {
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
    explorers.push(Explorer {
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
        explorers.push(Explorer {
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
        explorers.push(explorer);
    }

    explorers
}