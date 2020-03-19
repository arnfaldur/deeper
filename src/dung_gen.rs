extern crate rand;
extern crate ena;

use rand::{Rng};

use std::collections::{HashMap, HashSet};
use self::ena::unify::{UnifyKey, UnificationTable, InPlace};

pub const NOTHING : i32 = 0;
pub const WALL : i32 = 1;
pub const FLOOR : i32 = 2;
pub const WALL_NORTH : i32= 3;
pub const WALL_SOUTH : i32 = 4;
pub const WALL_EAST : i32 = 5;
pub const WALL_WEST : i32 = 6;
pub const DEBUG : i32 = 7;

pub struct DungGen {
    pub width      : i32,
    pub height     : i32,

    // The minimum width and height for a room
    pub room_min   : i32,
    // The maximum width and height are both
    // room_min + room_range
    pub room_range : i32,

    pub n_rooms : usize,

    // Used over the course of the algorithm,
    // made public to position player currently
    pub room_centers : Vec::<(i32, i32)>,
    // The result of the algorithm is stored here
    pub world : HashMap::<(i32, i32), i32>,
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
        DungGen { width: 100, height: 50, room_min: 4, room_range: 11, n_rooms: 10, room_centers: vec![], world: HashMap::<(i32, i32), i32>::new()}
    }

    pub fn width (mut self, width: i32)  -> DungGen { self.width = width; return self; }
    pub fn height(mut self, height: i32) -> DungGen { self.height = height; return self; }

    pub fn room_min  (mut self, room_min: i32)   -> DungGen { self.room_min = room_min; return self; }
    pub fn room_range(mut self, room_range: i32) -> DungGen { self.room_range = room_range; return self; }

    pub fn n_rooms(mut self, n_rooms: usize) -> DungGen { self.n_rooms = n_rooms; return self; }

    pub fn world(mut self, world : HashMap::<(i32, i32), i32>) -> DungGen { self.world = world; return self; }

    pub fn generate(mut self) -> DungGen {
        let mut rng = rand::thread_rng();

        self.room_centers = Vec::<(i32, i32)>::new();

        // This is how close to the edges of the map floors can be.
        // This parameter is needed since rooms are now simply the floor
        // and are then surrounded afterwards by walls (might change).
        let margin = 1;

        // n_rooms is 10 by default but should be set when constructing a room
        while self.room_centers.len() < self.n_rooms {

            // Step 1: Generate a random room in the world

            let xmin = rng.gen_range(margin, self.width  - (self.room_min + self.room_range) - margin);
            let ymin = rng.gen_range(margin, self.height - (self.room_min + self.room_range) - margin);

            let xmax = xmin + self.room_min + rng.gen_range(0, self.room_range);
            let ymax = ymin + self.room_min + rng.gen_range(0, self.room_range);

            // Step 2:  Check if the randomly generated room
            //          intersects with any previously generated room

            // Assume it does not
            let mut valid = true;
            // Check for intersection
            for x in xmin..=xmax {
                for y in ymin..=ymax {
                    if self.world.contains_key(&(x,y)) {
                        valid = false;
                        break;
                    }
                }
                if !valid { break; }
            }
            // If an intersection is found, go back to step 1
            if !valid { continue; }

            // Step 3: Paint the room into the world

            // Lay down floor
            for x in xmin..=xmax {
                for y in ymin..=ymax {
                    self.world.insert((x,y), FLOOR);
                }
            }

            // Set walls on the outside of the room
            for x in xmin-1..=xmax+1 {
                self.world.insert((x, ymin - 1), WALL);
                self.world.insert((x, ymax + 1), WALL);
            }
            for y in ymin-1..=ymax+1 {
                self.world.insert((xmin - 1, y), WALL);
                self.world.insert((xmax + 1, y), WALL);
            }

            // Add the center of the generated room to the list
            self.room_centers.push((xmin + (xmax - xmin) / 2, ymin + (ymax - ymin) / 2));
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

            // If there are none reamining, we are done.
            if remaining.len() == 0 { break; }

            // Select the pair of such rooms with the least distance between them.
            let mut to_connect = ((0,0), (0,0));
            let mut least_dist = std::i32::MAX;

            for ((a,b),(c,d)) in remaining {
                let dist = (a-c).abs() + (b-d).abs();
                if dist < least_dist {
                    least_dist = dist;
                    to_connect = ((a,b),(c,d));
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

            for x in x_start..=x_end+1 {
                if x <= x_end { self.world.insert((x, y_start), FLOOR); }
                if self.world.get(&(x, y_start+1)) == None { self.world.insert((x,y_start+1), WALL); }
                if self.world.get(&(x, y_start-1)) == None { self.world.insert((x,y_start-1), WALL); }
            }

            // And now make sure we iterate in the correct y direction as well.
            if y_start > y_end {
                let temp = y_start;
                y_start = y_end;
                y_end = temp;
            }

            for y in y_start..=y_end+1 {
                if y <= y_end { self.world.insert((x_end, y), FLOOR); }
                if self.world.get(&(x_end+1, y)) == None { self.world.insert((x_end+1,y), WALL);}
                if self.world.get(&(x_end-1, y)) == None { self.world.insert((x_end-1,y), WALL);}
            }

            // Finally mark these rooms as being connected
            let (r1, r2) = to_connect;
            comps.union(*keys.get(&r1).unwrap(), *keys.get(&r2).unwrap());
        }

        for x in 0..self.width {
            for y in 0..self.width {
                if self.world.get(&(x,y)) == None { self.world.insert((x,y), NOTHING); }
            }
        }

        let mut walls_north = vec!();
        let mut walls_south = vec!();
        let mut walls_east = vec!();
        let mut walls_west = vec!();

        for x in 1..self.width-1 {
            for y in 1..self.height-1 {
                let loc = (x,y);
                if *self.world.get(&loc).unwrap() == WALL {
                    let N = *self.world.get(&(x,y+1)).unwrap();
                    let S = *self.world.get(&(x,y-1)).unwrap();
                    let E = *self.world.get(&(x+1,y)).unwrap();
                    let W = *self.world.get(&(x-1,y)).unwrap();
                    //let NE = self.world.get(&(x+1,y+1));
                    //let NW = self.world.get(&(x-1,y+1));
                    //let SE = self.world.get(&(x+1,y-1));
                    //let SW = self.world.get(&(x-1,y-1));

                    if N == WALL || N == NOTHING {
                       if S == FLOOR && E == WALL && W == WALL {
                           walls_north.push(loc);
                           continue;
                       }
                    }
                    if S == WALL || S == NOTHING {
                        if N == FLOOR && E == WALL && W == WALL {
                            walls_south.push(loc);
                            continue;
                        }
                    }
                    if E == WALL || E == NOTHING {
                        if W == FLOOR && N == WALL && S == WALL {
                            walls_east.push(loc);
                            continue;
                        }
                    }
                    if W == WALL || W == NOTHING {
                        if E == FLOOR && N == WALL && S == WALL {
                            walls_west.push(loc);
                            continue;
                        }
                    }
                }
            }
        }

        for n in walls_north { self.world.insert(n, WALL_NORTH); }
        for s in walls_south { self.world.insert(s, WALL_SOUTH); }
        for e in walls_east  { self.world.insert(e, WALL_EAST); }
        for w in walls_west  { self.world.insert(w, WALL_WEST); }

        return self;
    }

    pub fn print(self) -> DungGen {
        for y in 0..self.height {
            for x in 0..self.width {
                match self.world.get(&(x,y)) {
                    None => print!("  "),
                    Some(&value) => match value {
                        WALL  => print!("# "),
                        FLOOR => print!(". "),
                        DEBUG => print!("X "),
                        _ => print!("? "),
                    }
                }
            }
            println!();
        }
        return self;
    }
}
