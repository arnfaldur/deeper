extern crate rand;
extern crate ena;

use rand::{Rng};

use std::collections::{HashMap, HashSet};
use self::ena::unify::{UnifyKey, UnificationTable, UnificationStore, InPlace};

pub enum TileKind {
    NOTHING,
    WALL,
    FLOOR,
    DEBUG,
}

pub const NOTHING : i32 = 0;
pub const WALL : i32 = 1;
pub const FLOOR : i32 = 2;
pub const DEBUG : i32 = 3;

pub struct DungGen {
    pub width      : i32,
    pub height     : i32,

    pub room_min   : i32,
    pub room_range : i32,

    pub n_rooms : usize,

    pub world : HashMap::<(i32, i32), i32>,
}

// (Internal screaming)
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
        DungGen { width: 100, height: 50, room_min: 4, room_range: 11, n_rooms: 10, world: HashMap::<(i32, i32), i32>::new()}
    }

    pub fn width(mut self, width: i32) -> DungGen { self.width = width; return self; }
    pub fn height(mut self, height: i32) -> DungGen { self.height = height; return self; }

    pub fn room_min(mut self, room_min: i32) -> DungGen { self.room_min = room_min; return self; }
    pub fn room_range(mut self, room_range: i32) -> DungGen { self.room_range = room_range; return self; }

    pub fn n_rooms(mut self, n_rooms: usize) -> DungGen { self.n_rooms = n_rooms; return self; }

    pub fn world(mut self, world : HashMap::<(i32, i32), i32>) -> DungGen { self.world = world; return self; }

    pub fn generate(mut self) -> DungGen {
        let mut rng = rand::thread_rng();

        let mut rooms = Vec::<((i32,i32),(i32,i32))>::new();

        while rooms.len() < self.n_rooms {
            let lu : (i32, i32) = (rng.gen_range(0, self.width - (self.room_min + self.room_range)), rng.gen_range(0, self.height - (self.room_min + self.room_range)));
            let rd : (i32, i32) = (lu.0 + self.room_min + rng.gen_range(0, self.room_range), lu.1 + self.room_min + rng.gen_range(0, self.room_range));

            let mut valid = true;
            for x in lu.0..=rd.0 {
                for y in lu.1..=rd.1 {
                    if self.world.contains_key(&(x,y)) {
                        valid = false;
                        break;
                    }
                }
                if !valid { break; }
            }
            if !valid { continue; }

            for x in lu.0..=rd.0 {
                for y in lu.1..=rd.1 {
                    match (x,y) {
                        (x,_) if x == lu.0 || x == rd.0 => self.world.insert((x,y), WALL),
                        (_,y) if y == lu.1 || y == rd.1 => self.world.insert((x,y), WALL),
                        _ => self.world.insert((x,y), FLOOR),
                    };
                }
            }
            rooms.push((lu, rd));
        }

        let mut room_centers = Vec::<(i32, i32)>::new();

        for ((xmin, ymin), (xmax, ymax)) in rooms {
            room_centers.push((xmin + (xmax - xmin) / 2, ymin + (ymax - ymin) / 2));
        }

        let mut keys = HashMap::<(i32, i32), UnitKey>::new();
        let mut comps: UnificationTable<InPlace<UnitKey>> = UnificationTable::new();
        let mut paths = HashSet::<((i32,i32), (i32,i32))>::new();

        for i in 0..room_centers.len() {
            keys.insert(room_centers[i], comps.new_key(()));
            paths.insert((room_centers[i], room_centers[i]));
        }

        loop {
            let mut remaining = Vec::<((i32, i32), (i32, i32))>::new();
            for r1 in &room_centers {
                for r2 in &room_centers {
                    if !comps.unioned(*keys.get(r1).unwrap(), *keys.get(r2).unwrap()) {
                        remaining.push((*r1, *r2));
                    }
                    //if !paths.contains(&(*r1,*r2)) {
                    //    remaining.push((*r1, *r2));
                    //}
                }
            }
            if remaining.len() == 0 { break; }

            let mut to_connect = ((0,0), (0,0));
            let mut least_dist = std::i32::MAX;

            for ((a,b),(c,d)) in remaining {
                let dist = (a-c).abs() + (b-d).abs();
                if dist < least_dist {
                    least_dist = dist;
                    to_connect = ((a,b),(c,d));
                } else if dist == least_dist && rand::random::<bool>() {
                    least_dist = dist;
                    to_connect = ((a,b),(c,d));
                }
            }

            let ((x0, y0), (x1, y1)) = to_connect;

            let (mut x_start, mut y_start, mut x_end, mut y_end) = (x0, y0, x1, y1);

            if x0 > x1 {
                // Not possible :( ... (x_start, y_start, x_end, y_end) = (x1,y1,x0,y0);
                x_start = x1;
                y_start = y1;
                x_end = x0;
                y_end = y0;
            }

            for x in x_start..=x_end {
                self.world.insert((x, y_start), FLOOR);
                if self.world.get(&(x, y_start+1)) == None { self.world.insert((x,y_start+1), WALL); }
                if self.world.get(&(x, y_start-1)) == None { self.world.insert((x,y_start-1), WALL); }
            }

            if y_start > y_end {
                // argh (y_start, y_end) = (y_end, y_start);
                let temp = y_start;
                y_start = y_end;
                y_end = temp;
            }

            for y in y_start..=y_end {
                self.world.insert((x_end, y), FLOOR);
                if self.world.get(&(x_end+1, y)) == None { self.world.insert((x_end+1,y), WALL);}
                if self.world.get(&(x_end-1, y)) == None { self.world.insert((x_end-1,y), WALL);}
            }

            let (r1, r2) = to_connect;

            comps.union(*keys.get(&r1).unwrap(), *keys.get(&r2).unwrap());

            //for r in &room_centers {
            //    if paths.contains(&(*r,r1)) {
            //        paths.insert((*r,r2));
            //        paths.insert((r2,*r));
            //    }
            //    if paths.contains(&(*r,r2)) {
            //        paths.insert((*r,r1));
            //        paths.insert((r1,*r));
            //    }
            //}
        }

        return self;
    }

    pub fn print(self) -> DungGen {
        for y in 0..self.height {
            for x in 0..self.width {
                match self.world.get(&(x,y)) {
                    None => print!("  "),
                    Some(value) => match value {
                        &WALL => print!("# "),
                        &FLOOR => print!(". "),
                        &DEBUG => print!("X "),
                        _ => print!("? "),
                    }
                }
            }
            println!();
        }
        return self;
    }
}
