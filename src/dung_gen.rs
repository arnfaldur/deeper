extern crate ena;
extern crate rand;

use rand::Rng;

use self::ena::unify::{InPlace, UnificationTable, UnifyKey};
use std::collections::HashMap;
use self::rand::thread_rng;
use crate::components::{TileType, WallDirection};

pub struct DungGen {
    pub width: i32,
    pub height: i32,

    // The minimum width and height for a room
    pub room_min: i32,
    // The maximum width and height are both
    // room_min + room_range
    pub room_range: i32,

    pub n_rooms: usize,

    // Used over the course of the algorithm,
    // made public to position player currently
    pub room_centers: Vec<(i32, i32)>,
    // The result of the algorithm is stored here
    pub world: HashMap<(i32, i32), TileType>,
}

// (Internal screaming)
// Needed for the Union-Find algorithm used (UnificationTable)
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct UnitKey(u32);

impl UnifyKey for UnitKey {
    type Value = ();
    fn index(&self) -> u32 {
        self.0
    }
    fn from_index(u: u32) -> UnitKey {
        UnitKey(u)
    }
    fn tag() -> &'static str {
        "UnitKey"
    }
}

// note(Jökull): There are better builder patterns
impl DungGen {
    pub fn new() -> DungGen {
        DungGen {
            width: 100,
            height: 50,
            room_min: 4,
            room_range: 11,
            n_rooms: 10,
            room_centers: vec![],
            world: HashMap::<(i32, i32), TileType>::new(),
        }
    }

    pub fn width(mut self, width: i32) -> DungGen {
        self.width = width;
        return self;
    }
    pub fn height(mut self, height: i32) -> DungGen {
        self.height = height;
        return self;
    }

    pub fn room_min(mut self, room_min: i32) -> DungGen {
        self.room_min = room_min;
        return self;
    }
    pub fn room_range(mut self, room_range: i32) -> DungGen {
        self.room_range = room_range;
        return self;
    }

    pub fn n_rooms(mut self, n_rooms: usize) -> DungGen {
        self.n_rooms = n_rooms;
        return self;
    }

    pub fn world(mut self, world: HashMap<(i32, i32), TileType>) -> DungGen {
        self.world = world;
        return self;
    }

    pub fn generate(mut self) -> DungGen {
        let mut rng = rand::thread_rng();

        self.room_centers = Vec::<(i32, i32)>::new();

        // This is how close to the edges of the map floors can be.
        // This parameter is needed since rooms are now simply the floor
        // and are then surrounded afterwards by walls (might change).
        let world_edge_size = 1;

        // n_rooms is 10 by default but should be set when constructing a room
        while self.room_centers.len() < self.n_rooms {
            // Step 1: Generate a random room in the world

            let x_min = rng.gen_range(
                world_edge_size..
                self.width - (self.room_min + self.room_range) - world_edge_size,
            );
            let y_min = rng.gen_range(
                world_edge_size..
                self.height - (self.room_min + self.room_range) - world_edge_size,
            );

            let x_max = x_min + self.room_min + rng.gen_range(0..self.room_range);
            let y_max = y_min + self.room_min + rng.gen_range(0..self.room_range);

            // Step 2:  Check if the randomly generated room
            //          intersects with any previously generated room

            // Assume it does not
            let mut valid = true;
            // Check for intersection
            'outer: for x in x_min..=x_max {
                for y in y_min..=y_max {
                    if self.world.contains_key(&(x, y)) {
                        valid = false;
                        break 'outer;
                    }
                }
            }
            // If an intersection is found, go back to step 1
            if !valid {
                continue;
            }

            // Step 3: Paint the room into the world

            // Lay down floor
            for x in x_min..=x_max {
                for y in y_min..=y_max {
                    self.world.insert((x, y), TileType::Floor);
                }
            }

            // Set walls on the outside of the room
            for x in x_min - 1..=x_max + 1 {
                self.world.insert((x, y_min - 1), TileType::Wall(None));
                self.world.insert((x, y_max + 1), TileType::Wall(None));
            }
            for y in y_min - 1..=y_max + 1 {
                self.world.insert((x_min - 1, y), TileType::Wall(None));
                self.world.insert((x_max + 1, y), TileType::Wall(None));
            }

            // Add the center of the generated room to the list
            self.room_centers
                .push((x_min + (x_max - x_min) / 2, y_min + (y_max - y_min) / 2));
        }

        // Step 4: Once all rooms are generated, add the centers as
        //         a list of keys in a UnificationTable

        let mut keys = HashMap::<(i32, i32), UnitKey>::new();
        let mut comps: UnificationTable<InPlace<UnitKey>> = UnificationTable::new();

        for i in 0..self.room_centers.len() {
            keys.insert(self.room_centers[i], comps.new_key(()));
        }

        // Step 5: Connect the pair of rooms that have the shortest distance
        //         between them and are in different components.

        loop {
            // Generate the remaining pairs of rooms that are
            // not yet connected by some path
            let mut remaining = Vec::<((i32, i32), (i32, i32))>::new();

            for r1 in &self.room_centers {
                for r2 in &self.room_centers {
                    if !comps.unioned(*keys.get(r1).unwrap(), *keys.get(r2).unwrap()) {
                        remaining.push((*r1, *r2));
                    }
                }
            }

            // If there are none remaining, we are done.
            if remaining.len() == 0 {
                break;
            }

            // Select the pair of such rooms with the least distance between them.
            let mut to_connect = ((0, 0), (0, 0));
            let mut least_dist = std::i32::MAX;

            for ((a, b), (c, d)) in remaining {
                let dist = (a - c).abs() + (b - d).abs();
                if dist < least_dist {
                    least_dist = dist;
                    to_connect = ((a, b), (c, d));
                }
            }

            // Create variables for the centers, where (x0, y0) is the first room
            // and (x1, y1) is the second room to be connected.
            let ((x0, y0), (x1, y1)) = to_connect;

            let (mut x_start, mut y_start, mut x_end, mut y_end) = (x0, y0, x1, y1);

            // For the algorithm to work correctly we have to make sure that x0 is less than x1
            if x0 > x1 {
                x_start = x1;
                y_start = y1;
                x_end = x0;
                y_end = y0;
            }

            for x in x_start..=x_end + 1 {
                if x <= x_end {
                    self.world.insert((x, y_start), TileType::Path);
                }
                if let None = self.world.get(&(x, y_start + 1)) {
                    self.world.insert((x, y_start + 1), TileType::Wall(None));
                }
                if let None = self.world.get(&(x, y_start - 1)) {
                    self.world.insert((x, y_start - 1), TileType::Wall(None));
                }
            }

            // And now make sure we iterate in the correct y direction as well.
            if y_start > y_end {
                let temp = y_start;
                y_start = y_end;
                y_end = temp;
            }

            for y in y_start..=y_end + 1 {
                if y <= y_end {
                    self.world.insert((x_end, y), TileType::Path);
                }
                if let None = self.world.get(&(x_end + 1, y)) {
                    self.world.insert((x_end + 1, y), TileType::Wall(None));
                }
                if let None = self.world.get(&(x_end - 1, y)) {
                    self.world.insert((x_end - 1, y), TileType::Wall(None));
                }
            }

            // Finally mark these rooms as being connected
            let (r1, r2) = to_connect;
            comps.union(*keys.get(&r1).unwrap(), *keys.get(&r2).unwrap());
        }

        // Determine the orientation of walls to assign the correct model and rotation
        let mut directed_walls = vec![];

        for (&(x, y), &wall_type) in self.world.iter() {
            let loc = (x, y);
            if let TileType::Wall(_) = wall_type {
                let n = *self.world.get(&(x, y + 1)).unwrap_or(&TileType::Wall(None));
                let w = *self.world.get(&(x - 1, y)).unwrap_or(&TileType::Wall(None));
                let s = *self.world.get(&(x, y - 1)).unwrap_or(&TileType::Wall(None));
                let e = *self.world.get(&(x + 1, y)).unwrap_or(&TileType::Wall(None));

                let ne = *self.world.get(&(x + 1, y + 1)).unwrap_or(&TileType::Wall(None));
                let nw = *self.world.get(&(x - 1, y + 1)).unwrap_or(&TileType::Wall(None));
                let se = *self.world.get(&(x + 1, y - 1)).unwrap_or(&TileType::Wall(None));
                let sw = *self.world.get(&(x - 1, y - 1)).unwrap_or(&TileType::Wall(None));

                for &(a, b, c, d, e, f, typ) in [
                    (s, e, n, w, ne, nw, TileType::Wall(Some(WallDirection::North))),
                    (e, n, w, s, sw, nw, TileType::Wall(Some(WallDirection::West))),
                    (n, w, s, e, se, sw, TileType::Wall(Some(WallDirection::South))),
                    (w, s, e, n, se, ne, TileType::Wall(Some(WallDirection::East))),
                ].iter() {
                    if (a == TileType::Floor || a == TileType::Path)
                        && b == TileType::Wall(None)
                        && c == TileType::Wall(None)
                        && d == TileType::Wall(None)
                        && e == TileType::Wall(None)
                        && f == TileType::Wall(None)
                    {
                        directed_walls.push((loc, typ));
                    }
                }
            }
        }

        for (loc, typ) in directed_walls {
            self.world.insert(loc, typ);
        }

        // Mark the rest of the world as consisting of nothing
        for x in 0..self.width {
            for y in 0..self.width {
                if let None = self.world.get(&(x, y)) {
                    self.world.insert((x, y), TileType::Nothing);
                }
            }
        }

        // Step 4.5: make a thing

        let mut rng = thread_rng();
        let ladder_loc = rng.gen_range(0..self.room_centers.len());
        self.world.insert(self.room_centers[ladder_loc], TileType::LadderDown);

        return self;
    }

    pub fn print(self) -> DungGen {
        for y in 0..self.height {
            for x in 0..self.width {
                match self.world.get(&(x, y)) {
                    None => print!("  "),
                    Some(&value) => match value {
                        TileType::Wall(None) => print!("# "),
                        TileType::Floor => print!(". "),
                        _ => print!("? "),
                    },
                }
            }
            println!();
        }
        return self;
    }
}
