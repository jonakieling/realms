
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
    RealmsList(Vec<RealmId>),
    RequestNewRealm,
    RequestRealm(RealmId),
    Realm(Realm),
    Explorer(Move),
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
    pub regions: Vec<Region>
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

            let how_many_particularities = rng2.sample(&Uniform::new_inclusive(0, 1));

            let particularities: Vec<Particularity> = rng2.sample_iter(&Uniform::new_inclusive(1, 3)).take(how_many_particularities).map(|number| {
                match number {
                    1 => Particularity::Town,
                    2 => Particularity::River,
                    _ => Particularity::Carravan
                }
            }).collect();

            let region = Region {
                id: region_id,
                terrain,
                particularities,
                buildings: vec![],
                mapped: false,
                resources: 10
            };
            region_id += 1;

            region
        }).collect();

        Island {
            regions
        }
    }

    pub fn plain() -> Island {
        Island {
            regions: vec![]
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Region {
    pub id: RegionId,
    pub terrain: Terrain,
    pub particularities: Vec<Particularity>,
    pub buildings: Vec<String>,
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
    Camp,
    Gear(Gear)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Expedition {
    pub explorers: Vec<Explorer>
}

impl Expedition {
    pub fn new() -> Expedition {
        Expedition {
            explorers: vec![
                Explorer { id: 0, traits: vec![ExplorerTrait::Ranger], region: None, inventory: vec![Gear::SurvivalKit] },
                Explorer { id: 1, traits: vec![ExplorerTrait::Cartographer], region: None, inventory: vec![Gear::ClimbingGear] },
                Explorer { id: 2, traits: vec![ExplorerTrait::Engineer], region: None, inventory: vec![Gear::HotAirBalloon] },
                Explorer { id: 3, traits: vec![ExplorerTrait::Sailor], region: None, inventory: vec![Gear::Boat] }
            ]
        }
    }

    pub fn plain() -> Expedition {
        Expedition {
            explorers: vec![]
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Explorer {
    pub id: ExplorerId,
    pub traits: Vec<ExplorerTrait>,
    pub region: Option<RegionId>,
    pub inventory: Vec<Gear>
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
            for explorer_trait in &self.traits {
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
pub enum Gear {
    SurvivalKit,
    HotAirBalloon,
    Boat,
    ClimbingGear
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