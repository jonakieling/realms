
use std::ops::Add;

#[derive(Debug)]
pub struct Cube {
	x: isize,
	y: isize,
	z: isize
}

impl Cube {
	pub fn new(x: isize, y: isize, z: isize) -> Cube {
		Cube { x, y, z }
	}
}

impl<'a> Add for &'a Cube {
	type Output = Cube;

	fn add(self, other: &'a Cube) -> Cube {
		Cube::new(self.x + other.x, self.y + other.y, self.z + other.z)
	}
}

impl PartialEq for Cube {
	fn eq(&self, other: &Cube) -> bool {
		self.x == other.x && self.y == other.y && self.z == other.z
	}
}

#[derive(Debug)]
pub struct Hex {
	pub id: usize,
    pub cube: Cube,
    pub offset: (usize, usize),
    pub neighbors: Vec<usize>
}

pub fn hexes(rows: usize, cols: usize) -> Vec<Hex> {
	let mut id = 0;

	let mut hexes = vec![];
	for row in 0..rows {
		for col in 0..cols {
			let cube = oddr_to_cube(col as isize, row as isize);
		    hexes.push(Hex {
		    	id,
		    	cube,
		    	offset: (col, row),
		    	neighbors: vec![]
		    });
		    id += 1;
		}
	}

	let mut neighbors = vec![];
	for hex in &hexes {
		for neighbor in cube_neighbors(&hex.cube) {
			for possible_neighbor in &hexes {
			    if possible_neighbor.cube == neighbor {
			        neighbors.push((hex.id, possible_neighbor.id));
			    }
			}
		}
	}

	for (id, neighbor) in neighbors {
		// id is taken from an existing hex and id to index matches on the hexes vec
	    hexes[id].neighbors.push(neighbor);
	}

	hexes

}

pub enum CubeDirection {
	TopLeft,
	TopRight,
	Left,
	Right,
	BottomLeft,
	BottomRight
}

impl CubeDirection {
	pub fn value(&self) -> Cube {
		match self {
			CubeDirection::TopLeft => Cube::new(0, 1, -1),
			CubeDirection::TopRight => Cube::new(1, 0, -1),
			CubeDirection::Left => Cube::new(-1, 1, 0),
			CubeDirection::Right => Cube::new(1, -1, 0),
			CubeDirection::BottomLeft => Cube::new(0, -1, 1),
			CubeDirection::BottomRight => Cube::new(-1, 0, 1)
		}
	}

	pub fn all() -> Vec<CubeDirection> {
		vec![
			CubeDirection::TopLeft,
			CubeDirection::TopRight,
			CubeDirection::Left,
			CubeDirection::Right,
			CubeDirection::BottomLeft,
			CubeDirection::BottomRight
			]
	}
}

pub fn cube_neighbor(cube: &Cube, direction: CubeDirection) -> Cube {
    cube + &direction.value()
}

pub fn cube_neighbors(cube: &Cube) -> Vec<Cube> {
	let mut neighbors = vec![];

	for direction in CubeDirection::all() {
	    neighbors.push(cube_neighbor(cube, direction));
	}

	neighbors
}


pub fn oddr_to_cube(row: isize, col: isize) -> Cube {
    let x = col - (row + (row & 1)) / 2;
    let z = row;
    let y = -x-z;
    Cube::new(x, y, z)
}