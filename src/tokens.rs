
use utility::SelectionStorage;
use std::fmt;
use rand::{thread_rng, distributions::Uniform, Rng};

pub type ClientId = usize;
pub type RealmId = usize;
pub type RegionId = usize;
pub type ExplorerId = usize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RealmsProtocol {
    Register,
    Connect(ClientId),
    RequestRealmsList,
    RealmsList(SelectionStorage<RealmId>),
    RequestNewRealm,
    RequestRealm(RealmId),
    Realm(Realm),
    Explorer(Move),
    DropItem(RealmId, RegionId, ExplorerId, Item),
    PickItem(RealmId, RegionId, ExplorerId, Item),
    Quit,
    Void
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Move {
    ChangeRegion(RealmId, RegionId, ExplorerId),
    Action(RealmId, RegionId, ExplorerId, ExplorerAction)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExplorerAction {
    Build,
    Hunt,
    Sail,
    Map,
    Wait
}

impl fmt::Display for RealmsProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Island {
    pub regions: SelectionStorage<Region>
}

impl Island {
    pub fn new() -> Island {
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

                    rng2.sample_iter(&Uniform::new_inclusive(1, 8)).take(how_many_particularities).map(|number| {
                        match number {
                            1 => Particularity::Town,
                            2 => Particularity::River,
                            3 => Particularity::Cliffs,
                            4 => Particularity::Cliffs,
                            5 => Particularity::Cliffs,
                            6 => Particularity::Island,
                            7 => Particularity::Island,
                            _ => Particularity::Carravan
                        }
                    }).collect()
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
                    }).collect()
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
                    }).collect()
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
                    }).collect()
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

        Island {
            regions: SelectionStorage::new_from(&regions)
        }
    }

    pub fn plain() -> Island {
        Island {
            regions: SelectionStorage::new()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Region {
    pub id: RegionId,
    pub terrain: Terrain,
    pub particularities: SelectionStorage<Particularity>,
    pub buildings: SelectionStorage<String>,
    pub mapped: bool,
    pub resources: usize
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", self.id, self.terrain)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Terrain {
    Coast,
    Planes,
    Forest,
    Mountain
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Particularity {
	Town,
	River,
	Carravan,
    Merchant,
    Camp,
    Item(Item),
    Canyon,
    Bolders,
    Grasland,
    Creek,
    Grove,
    Cliffs,
    Island,
    Lake,
    Pond,
    Clearing
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Expedition {
    pub explorers: SelectionStorage<Explorer>
}

impl Expedition {
    pub fn new() -> Expedition {
        Expedition {
            explorers: SelectionStorage::new_from(&vec![
                Explorer { id: 0, traits: SelectionStorage::new_from(&vec![ExplorerTrait::Ranger]), region: None, inventory: SelectionStorage::new_from(&vec![Equipment::SurvivalKit]) },
                Explorer { id: 1, traits: SelectionStorage::new_from(&vec![ExplorerTrait::Cartographer]), region: None, inventory: SelectionStorage::new_from(&vec![Equipment::ClimbingGear]) },
                Explorer { id: 2, traits: SelectionStorage::new_from(&vec![ExplorerTrait::Engineer]), region: None, inventory: SelectionStorage::new_from(&vec![Equipment::HotAirBalloon]) },
                Explorer { id: 3, traits: SelectionStorage::new_from(&vec![ExplorerTrait::Sailor]), region: None, inventory: SelectionStorage::new_from(&vec![Equipment::Boat]) }
            ])
        }
    }

    pub fn plain() -> Expedition {
        Expedition {
            explorers: SelectionStorage::new()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Explorer {
    pub id: ExplorerId,
    pub traits: SelectionStorage<ExplorerTrait>,
    pub region: Option<RegionId>,
    pub inventory: SelectionStorage<Equipment>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExplorerTrait {
    Ranger,
    Cartographer,
    Engineer,
    Sailor
}

impl fmt::Display for ExplorerTrait {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExplorerTrait::Ranger => write!(f, "Ranger"),
            ExplorerTrait::Cartographer => write!(f, "Cartographer"),
            ExplorerTrait::Engineer => write!(f, "Engineer"),
            ExplorerTrait::Sailor => write!(f, "Sailor")
        }
    }
}

impl Explorer {
    pub fn actions(&self) -> Vec<ExplorerAction> {
        let mut actions = vec![];
        if self.region.is_some() {
            for explorer_trait in self.traits.iter() {
                match explorer_trait {
                    ExplorerTrait::Ranger => actions.push(ExplorerAction::Hunt),
                    ExplorerTrait::Cartographer => actions.push(ExplorerAction::Map),
                    ExplorerTrait::Engineer => actions.push(ExplorerAction::Build),
                    ExplorerTrait::Sailor => actions.push(ExplorerAction::Sail)
                }
            }
        }
        actions
    }
}

impl fmt::Display for Explorer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Equipment {
    SurvivalKit,
    HotAirBalloon,
    Boat,
    ClimbingGear
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Item {
    Equipment(Equipment),
    Text(String)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Realm {
    pub island: Island,
    pub expedition: Expedition,
    pub id: RealmId,
    pub age: usize
}

impl Realm {
    pub fn plain(id: RealmId) -> Realm {
        Realm {
            island: Island::plain(),
            expedition: Expedition::plain(),
            id,
            age: 0
        }
    }

    pub fn new(id: RealmId) -> Realm {
        Realm {
            island: Island::new(),
            expedition: Expedition::new(),
            id,
            age: 0
        }
    }
}