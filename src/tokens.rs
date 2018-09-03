
use realms::RealmStrategy;
use utility::*;
use std::fmt;
use std::cmp;
use std::hash::{Hash, Hasher};

use uuid::Uuid;

pub type ClientId = Uuid;
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
        match self {
            RealmsProtocol::Connect(client) => write!(f, "Connect({})", client.to_string()),
            _ => write!(f, "{:?}", self)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Realm {
    pub island: Island,
    pub expedition: Expedition,
    pub id: RealmId,
    pub age: usize,
    pub title: String,
    pub story: String,
    pub objectives: Vec<RealmObjective>,
    pub completed: Vec<RealmObjective>,
    pub done: bool
}

impl Realm {
    pub fn new(id: RealmId) -> Realm {
        Realm {
            island: Island::new(),
            expedition: Expedition::new(),
            id,
            age: 0,
            title: "a realm".to_string(),
            story: "pure nihilism.".to_string(),
            objectives: vec![],
            completed: vec![],
            done: false
        }
    }
}

pub trait LazyRealmAccess<'a> {
    fn region(&'a mut self, region: RegionId) -> Option<&'a mut Region>;
    fn region_explorer(&'a mut self, region: RegionId, explorer: ExplorerId) -> Option<&'a mut Explorer>;
    fn explorer_region(&'a mut self, explorer: ExplorerId) -> Option<&'a mut Region>;
    fn explorer(&'a mut self, explorer: ExplorerId) -> Option<&'a mut Explorer>;
}

impl<'a> LazyRealmAccess<'a> for Option<&'a mut RealmStrategy> {
    fn explorer(&'a mut self, explorer: ExplorerId) -> Option<&'a mut Explorer> {
        match self {
            Some(RealmStrategy {variant, realm, template}) => {
                realm.expedition.explorers.storage_mut().get_mut(explorer)
            },
            None => None,
        }
    }

    fn region_explorer(&'a mut self, region: RegionId, explorer: ExplorerId) -> Option<&'a mut Explorer> {
        match self {
            Some(RealmStrategy {variant, realm, template}) => {
                if let Some(explorer) = realm.expedition.explorers.storage_mut().get_mut(explorer) {
                    if let Some(explorer_region) = explorer.region {
                        if region == explorer_region {
                            Some(explorer)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            None => None,
        }
    }

    fn explorer_region(&'a mut self, explorer: ExplorerId) -> Option<&'a mut Region> {
        match self {
            Some(RealmStrategy {variant, realm, template}) => {
                if let Some(explorer) = realm.expedition.explorers.storage_mut().get_mut(explorer) {
                    if let Some(region) = explorer.region {
                        realm.island.regions.storage_mut().get_mut(&region)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            None => None,
        }
    }

    fn region(&'a mut self, region: RegionId) -> Option<&'a mut Region> {
        match self {
            Some(RealmStrategy {variant, realm, template}) => {
                realm.island.regions.storage_mut().get_mut(&region)
            },
            None => None,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub enum RealmObjective {
    EmbarkExplorers
}

impl fmt::Display for RealmObjective {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RealmObjective::EmbarkExplorers => write!(f, "embark all explorers.")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Island {
    pub regions: SelectionHashMap<Region>
}

impl Island {
    pub fn new() -> Island {
        Island {
            regions: SelectionHashMap::new()
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
    pub resources: usize,
    pub sight: RegionVisibility,
    pub neighbors: Vec<RegionId>,
    pub hex_offset_coords: (usize, usize)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RegionVisibility {
    None,
    Partial,
    Complete,
    Live
}

impl cmp::PartialOrd for Region {
    fn partial_cmp(&self, other: &Region) -> Option<cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl cmp::Ord for Region {
    fn cmp(&self, other: &Region) -> cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl cmp::PartialEq for Region {
    fn eq(&self, other: &Region) -> bool {
        self.id == other.id
    }
}

impl cmp::Eq for Region { }

impl Hash for Region {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", self.id, self.terrain)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Terrain {
    Coast,
    Planes,
    Forest,
    Mountain
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, Copy)]
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
    Ship,
    Queen,
    Farmers,
    Lighthouse,
    Library,
    Castle,
    Fortress,
    Haven,
    Character
    // todo add more for plot and story
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Character {
    name: String,
    text: String,
    dialog: Vec<String>
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, Copy)]
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