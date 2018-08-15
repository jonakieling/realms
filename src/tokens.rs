
use rand::{thread_rng, distributions::Uniform, Rng};

#[derive(Serialize, Deserialize, Debug)]
pub struct Island {
    pub tiles: Vec<Tile>
}

impl Island {
    pub fn new() -> Island {
        let mut rng = thread_rng();
        let mut rng2 = thread_rng();
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

            Tile {
                terrain,
                particularities
            }
        }).collect();

        Island {
            tiles
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tile {
    pub terrain: Terrain,
    pub particularities: Vec<Particularity>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Terrain {
    Coast,
    Planes,
    Forest,
    Mountain
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Particularity {
	Town,
	River,
	Carravan
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Expedition {
    pub explorers: Vec<Explorer>,
    pub gear: Vec<Gear>
}

impl Expedition {
    pub fn new() -> Expedition {
        Expedition {
            explorers: vec![],
            gear: vec![]
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Explorer {
    Ranger,
    Cartographer,
    Engineer,
    Sailor
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Gear {
    Tent,
    Tools
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Realm {
    pub island: Island,
    pub expedition: Expedition,
    pub id: usize
}