#[derive(Serialize, Deserialize, Debug)]
pub struct Island {
    pub tiles: Vec<Tile>
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