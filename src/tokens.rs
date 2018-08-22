
use utility::SelectionStorage;
use std::fmt;

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
    DropEquipment(RealmId, RegionId, ExplorerId, Equipment),
    PickEquipment(RealmId, RegionId, ExplorerId, Equipment),
    InvestigateParticularity(RealmId, RegionId, ExplorerId, Particularity),
    ForgetParticularity(RealmId, RegionId, ExplorerId, Particularity),
    Quit,
    Void
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Move {
    ChangeRegion(RealmId, RegionId, ExplorerId),
    Action(RealmId, RegionId, ExplorerId, ExplorerAction)
}

impl fmt::Display for RealmsProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Realm {
    pub island: Island,
    pub expedition: Expedition,
    pub id: RealmId,
    pub age: usize
}

impl Realm {
    pub fn new(id: RealmId) -> Realm {
        Realm {
            island: Island::new(),
            expedition: Expedition::new(),
            id,
            age: 0
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Island {
    pub regions: SelectionStorage<Region>
}

impl Island {
    pub fn new() -> Island {
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub enum Particularity {
	Town,
	River,
	Carravan,
    Merchant,
    Camp,
    Item(Equipment),
    Canyon,
    Bolders,
    Grasland,
    Creek,
    Grove,
    Cliffs,
    Island,
    Lake,
    Pond,
    Clearing,
    Ship
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Expedition {
    pub explorers: SelectionStorage<Explorer>
}

impl Expedition {
    pub fn new() -> Expedition {
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
    pub inventory: SelectionStorage<ExplorerItem>
}

impl Explorer {
    pub fn trait_actions(&self) -> Vec<ExplorerAction> {
        let mut actions = vec![];
        if self.region.is_some() {
            for explorer_trait in self.traits.iter() {
                match explorer_trait {
                    ExplorerTrait::Ranger => actions.push(ExplorerAction::Hunt),
                    ExplorerTrait::Cartographer => actions.push(ExplorerAction::Map),
                    ExplorerTrait::Builder => actions.push(ExplorerAction::Build),
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
pub enum ExplorerAction {
    Build,
    Hunt,
    Sail,
    Map,
    Wait
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub enum ExplorerTrait {
    Ranger,
    Cartographer,
    Builder,
    Sailor
}

impl fmt::Display for ExplorerTrait {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExplorerTrait::Ranger => write!(f, "Ranger"),
            ExplorerTrait::Cartographer => write!(f, "Cartographer"),
            ExplorerTrait::Builder => write!(f, "Engineer"),
            ExplorerTrait::Sailor => write!(f, "Sailor")
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub enum Equipment {
    Pots,
    Tinder,
    Firewood(usize),
    Coal(usize),
    Gold(usize),
    Coins(usize),
    Tools,
    Flint,
    Wax,
    SealStamp,
    Blankets,
    Herbs(usize),
    Food(usize),
    Pipe,
    Telescope,
    Compass,
    Rope,
    Parchment(usize),
    Map,
    Knife,
    Spear,
    Bow,
    Arrows(usize),
    Canoe,
    Raft
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub enum ExplorerItem {
    Equipment(Equipment),
    Particularity(RegionId, Particularity),
    Message(String)
}