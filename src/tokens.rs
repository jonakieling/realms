
use std::fmt;
use rand::{thread_rng, distributions::Uniform, Rng};

#[derive(Serialize, Deserialize, Debug)]
pub enum RealmsProtocol {
    Register,
    Connect(usize),
    RequestRealmsList,
    RealmsList(Vec<usize>),
    RequestNewRealm,
    RequestRealm(usize),
    Realm(Realm),
    Move(Move),
    Quit,
    NotImplemented
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Move {
    ChangeLocation(usize, usize)
}

impl fmt::Display for RealmsProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Island {
    pub tiles: Vec<Tile>
}

impl Island {
    pub fn new() -> Island {
        let mut rng = thread_rng();
        let mut rng2 = thread_rng();
        let mut tile_id = 0;
        let tiles: Vec<Tile> = rng.sample_iter(&Uniform::new_inclusive(1, 4)).take(19).map(|number| {
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

            let tile = Tile {
                id: tile_id,
                terrain,
                particularities
            };
            tile_id += 1;

            tile
        }).collect();

        Island {
            tiles
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tile {
    pub id: usize,
    pub terrain: Terrain,
    pub particularities: Vec<Particularity>
}

impl fmt::Display for Tile {
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
	Carravan
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Expedition {
    pub explorers: Vec<Explorer>,
    pub gear: Vec<Gear>
}

impl Expedition {
    pub fn new() -> Expedition {
        Expedition {
            explorers: vec![Explorer::Ranger, Explorer::Cartographer, Explorer::Engineer, Explorer::Sailor],
            gear: vec![Gear::Tent, Gear::Tools]
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Explorer {
    Ranger,
    Cartographer,
    Engineer,
    Sailor
}

impl fmt::Display for Explorer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Gear {
    Tent,
    Tools
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Realm {
    pub island: Island,
    pub expedition: Expedition,
    pub client_locations: Vec<(usize, usize)>,
    pub id: usize,
    pub age: usize
}