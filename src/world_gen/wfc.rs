use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::iter::{FromIterator, Map};
use std::ops::{Index, IndexMut, Range};

use bit_set::BitSet;
use cgmath::Vector2;
use image::{ImageBuffer, Pixel};
use itertools::Itertools;
use rand::prelude::*;

use crate::world_gen::grid::{Grid, V2i, V2u};

#[allow(dead_code)]
const SQUARE_NEIGHBOURHOOD: [V2i; 9] = [
    V2i::new(-1, -1),
    V2i::new(0, -1),
    V2i::new(1, -1),
    V2i::new(-1, 0),
    V2i::new(0, 0),
    V2i::new(1, 0),
    V2i::new(-1, 1),
    V2i::new(0, 1),
    V2i::new(1, 1),
];
#[allow(dead_code)]
const CROSS_NEIGHBOURHOOD: [V2i; 5] = [
    V2i::new(0, -1),
    V2i::new(-1, 0),
    V2i::new(0, 0),
    V2i::new(1, 0),
    V2i::new(0, 1),
];
#[allow(dead_code)]
const SMALL_SQUARE_NEIGHBOURHOOD: [V2i; 4] = [
    V2i::new(0, 0),
    V2i::new(1, 0),
    V2i::new(0, 1),
    V2i::new(1, 1),
];

// impl<T: Display> Display for Grid<T> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result<T, _> { todo!() }
// }

impl<P: Pixel + 'static> From<&Grid<P>> for ImageBuffer<P, Vec<P::Subpixel>> {
    fn from(grid: &Grid<P>) -> Self {
        let mut result = ImageBuffer::new(grid.size.x as u32, grid.size.y as u32);
        for (y, row) in result.rows_mut().enumerate() {
            for (x, pixel) in row.enumerate() {
                *pixel = grid[V2u { y, x }];
            }
        }
        return result;
    }
}

/// A container that
#[derive(Debug)]
struct EntropyHierarchy {
    pub hierarchy: BTreeMap<usize, BitSet>,
}

impl EntropyHierarchy {
    fn new() -> Self {
        Self {
            hierarchy: BTreeMap::new(),
        }
    }
    fn add(&mut self, value: usize, entropy: usize) {
        if !self.hierarchy.entry(entropy).or_default().insert(value) {
            println!(
                "adding {} to {}, in hierarchy {:?}",
                value, entropy, self.hierarchy
            );
            panic!();
        }
    }

    fn reduce_entropy(&mut self, value: usize, original_entropy: usize, reduced_entropy: usize) {
        if let Some(true) = self
            .hierarchy
            .get_mut(&original_entropy)
            .map(|e| e.remove(value))
        {
            // println!(
            //     "Reducing entropy of {} from {} to {}",
            //     value, original_entropy, reduced_entropy
            // );
        } else {
            println!(
                "value {} missing from entropy class {}, going to {} in hierarchy {:?}",
                value, original_entropy, reduced_entropy, self.hierarchy
            );
            panic!();
        }
        self.add(value, reduced_entropy);
    }
    fn get_lowest_entropy(&mut self) -> (&usize, &mut BitSet) {
        self.hierarchy.iter_mut().find(|(i, _)| **i > 1).unwrap()
    }
    fn cleanup(&mut self) { self.hierarchy.retain(|_, set| !set.is_empty()); }
    fn is_converged(&self) -> bool {
        for layer in self.hierarchy.iter() {
            if *layer.0 > 1 {
                return false;
            }
        }
        return true;
    }
    #[allow(dead_code)]
    fn print<T>(&self, map: &Grid<T>) {
        let mut display = Grid::<usize>::new();
        display.resize(map.size, 0);
        println!("Entropy map:");
        for (entropy, bit_set) in self.hierarchy.iter() {
            for bit in bit_set.iter() {
                display.buf[bit] = *entropy;
            }
        }
        let mut i = 0;
        for _ in 0..display.size.y {
            for _ in 0..display.size.x {
                print!("{}, ", display.buf[i]);
                i += 1;
            }
            print!("\n");
        }
    }
}

// struct Constraints<T> {
//     color_locations: HashMap<T, BitSet>,
//     adjacencies: HashMap<(T, V2i), BitSet>,
// }

#[derive(Clone, Hash, Debug)]
struct Tile<T>(Vec<Option<T>>);

impl<T: Eq> Tile<T> {
    fn is_adjacent(
        &self,
        inverse_neighbourhood: &HashMap<V2i, usize>,
        intersection: &Vec<V2i>,
        offset: V2i,
        other: &Tile<T>,
    ) -> bool {
        for inter in intersection.iter() {
            if self.0[inverse_neighbourhood[inter]]
                != other.0[inverse_neighbourhood[&(inter - offset)]]
            {
                return false;
            }
        }
        return true;
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
struct TileHash(u64);

// a hash of a Tile<T>
#[derive(Debug)]
struct InputMap<T>(Vec<T>);

// a set of T that can be indexed like the input grid
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
struct InputIndex(usize);

// an index into the InputMap
impl<T> Index<InputIndex> for InputMap<T> {
    type Output = T;
    fn index(&self, index: InputIndex) -> &Self::Output { &self.0[index.0] }
}

impl<T> IndexMut<InputIndex> for InputMap<T> {
    fn index_mut(&mut self, index: InputIndex) -> &mut Self::Output { &mut self.0[index.0] }
}

#[derive(Debug)]
struct TileMap<T>(Vec<T>);

// a set of things mapped to each unique tile
impl<T> TileMap<T> {
    fn range(&self) -> Map<Range<usize>, fn(usize) -> TileIndex> {
        (0..self.0.len()).map(|i| TileIndex(i))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
struct TileIndex(usize);

// an index into the TileMap
impl<T> Index<TileIndex> for TileMap<T> {
    type Output = T;
    fn index(&self, index: TileIndex) -> &Self::Output { &self.0[index.0] }
}

impl<T> IndexMut<TileIndex> for TileMap<T> {
    fn index_mut(&mut self, index: TileIndex) -> &mut Self::Output { &mut self.0[index.0] }
}

pub fn wfc<T: Copy + Eq + Hash + Debug>(
    input: Grid<T>,
    neighbourhood: Vec<V2i>,
    output_size: V2u,
) -> Vec<Grid<Result<T, usize>>> {
    let mut rng = rand::prelude::StdRng::seed_from_u64(1337);

    let inverse_neighbourhood: HashMap<V2i, usize> = neighbourhood
        .iter()
        .enumerate()
        .map(|(i, off)| (*off, i))
        .collect();
    println!("{:?}", inverse_neighbourhood);

    // a tile for each point in the input
    let input_tiles: InputMap<Tile<T>> = InputMap(
        (0..input.buf.len())
            .map(|i| {
                let index_2d = input.to_2d_index(i as isize).unwrap();
                Tile(
                    neighbourhood
                        .iter()
                        .map(|&offset| input.get(index_2d + offset).map(|c| *c))
                        .collect(),
                )
            })
            .collect(),
    );
    // a mapping from an input index to a hash of a tile
    let input_tile_hashes: InputMap<TileHash> = InputMap(
        input_tiles
            .0
            .iter()
            .map(|tile| {
                let mut hasher = DefaultHasher::new();
                tile.hash(&mut hasher);
                return TileHash(hasher.finish());
            })
            .collect(),
    );
    // a set of indexes into an InputMap for each unique tile
    let tiles: TileMap<InputIndex> = TileMap(
        input_tile_hashes
            .0
            .iter()
            .enumerate()
            .unique_by(|(_, &tile)| tile)
            .map(|(i, _)| InputIndex(i))
            .collect(),
    );

    let hash_to_tile_index: HashMap<TileHash, TileIndex> = HashMap::from_iter(
        tiles
            .0
            .iter()
            .enumerate()
            .map(|(tile_id, input_id)| (input_tile_hashes[*input_id], TileIndex(tile_id))),
    );
    // a mapping from input indices to indexes of a TileMap
    let input_to_tiles: InputMap<TileIndex> = InputMap(
        input_tile_hashes
            .0
            .iter()
            .map(|hash| hash_to_tile_index[hash])
            .collect(),
    );
    println!("inputs to tiles {:?}", input_to_tiles);
    // the inverse of the above
    let tiles_to_inputs: TileMap<Vec<InputIndex>> = input_to_tiles.0.iter().enumerate().fold(
        TileMap(tiles.0.iter().map(|_| Vec::new()).collect()),
        |mut result, (i, &tile_index)| {
            result[tile_index].push(InputIndex(i));
            return result;
        },
    );
    println!("tiles to inputs {:?}", tiles_to_inputs);
    let mut adjacencies: HashMap<V2i, TileMap<BitSet>> = neighbourhood
        .iter()
        .filter(|offset| **offset != V2i::new(0, 0))
        .fold(HashMap::new(), |mut result, &offset| {
            // This iterates over the offsets of the chosen neighbourhood
            // and creates a mapping from tiles to
            // let boi: Vec<V2i> = HashSet::from_iter(neighbourhood.iter()).intersection(&HashSet::from_iter(
            //     neighbourhood.iter().map(|of| of + offset),
            // )).collect();
            result
                .entry(offset)
                .or_insert(TileMap(Vec::new()))
                .0
                .extend((0..tiles.0.len()).map(|tile_index| {
                    BitSet::from_iter(
                        tiles_to_inputs
                            .0
                            .get(tile_index)
                            .unwrap()
                            .iter()
                            .filter_map(|&InputIndex(input_id)| {
                                input
                                    .to_1d_index(
                                        input.to_2d_index(input_id as isize).unwrap() + offset,
                                    )
                                    .map(|input_neighbour| {
                                        input_to_tiles[InputIndex(input_neighbour as usize)].0
                                    })
                            }),
                    )
                }));
            return result;
        });
    println!("adjacencies v1 {:?}", adjacencies);
    for (offset, bit_set) in adjacencies.iter_mut() {
        let shifted: Vec<V2i> = neighbourhood.iter().map(|off| off + offset).collect();
        let intersection: Vec<V2i> = neighbourhood
            .iter()
            .filter(|off| shifted.contains(off))
            .map(|off| *off)
            .collect();

        for i in (0..tiles.0.len()).map(|x| TileIndex(x)) {
            for j in (0..tiles.0.len()).map(|x| TileIndex(x)) {
                if i != j {
                    let a = &input_tiles[tiles[i]];
                    let b = &input_tiles[tiles[j]];
                    if a.is_adjacent(&inverse_neighbourhood, &intersection, *offset, b) {
                        bit_set[i].insert(j.0);
                    }
                }
            }
        }
    }
    println!("adjacencies v2 {:?}", adjacencies);

    // let boi: Vec<V2i> = HashSet::from_iter(neighbourhood.iter()).intersection(&HashSet::from_iter(
    //     neighbourhood.iter().map(|of| of + offset),
    // )).collect();
    let mut wave_map = Grid::new();
    wave_map.resize(output_size, BitSet::from_iter(0..tiles.0.len()));
    // TODO: consider using this to make reversable steps
    //let mut wave_map_backup = wave_map.copy();
    for i in tiles.range() {
        println!("tile {}", i.0);

        for j in 0..4 {
            let thing = input_tiles[tiles[i]].0[j];
            if j == 2 {
                println!();
            }
            print!(
                "{}",
                if thing == input_tiles[InputIndex(0)].0[0] {
                    "W"
                } else if thing == input_tiles[InputIndex(0)].0[3] {
                    "b"
                } else {
                    "."
                }
            );
        }
        println!();
        // println!("{:?}", input_tiles[tiles[i]]);
    }
    println!("{:?}, ", SMALL_SQUARE_NEIGHBOURHOOD);
    for i in 0..tiles.0.len() {
        print!("tile: {} -> ", i);
        for offset in SMALL_SQUARE_NEIGHBOURHOOD.iter() {
            if let Some(boii) = adjacencies.get(offset).and_then(|me| me.0.get(i)) {
                print!("{:?}, ", boii);
            } else {
                print!("{{{}}}, ", i);
            }
        }
        println!();
    }

    let mut entropy_hierarchy = EntropyHierarchy::new();
    for (i, set) in wave_map.buf.iter().enumerate() {
        entropy_hierarchy.add(i, set.len());
    }
    //let mut entropy_hierarchy_backup = entropy_hierarchy.copy();

    let mut result = Vec::new();
    while !entropy_hierarchy.is_converged() {
        println!("---------- iteration ----------");
        // collapse superposition
        let (&least_entropy, bottom) = entropy_hierarchy.get_lowest_entropy();
        let collapsing_output_index = bottom.iter().choose(&mut rng).unwrap();
        // println!(
        //     "collapsing: {:?} {:?} with entropy {}",
        //     to_2d_index(collapse_index as isize, wave_map.size.x),
        //     collapse_index,
        //     least_entropy
        // );
        print!("choosing tile: ");
        let chosen_tile = {
            let mut chosen_tile = TileIndex(0);
            'find: while let Some(tile) = wave_map.buf[collapsing_output_index]
                .iter()
                .choose(&mut rng)
                .map(|x| TileIndex(x))
            {
                print!("{}, ", tile.0);
                for (&offset, constraint) in adjacencies.iter() {
                    if constraint[tile].is_empty()
                        != wave_map
                            .to_1d_index(
                                wave_map
                                    .to_2d_index(collapsing_output_index as isize)
                                    .unwrap()
                                    + offset,
                            )
                            .is_none()
                    {
                        wave_map.buf[collapsing_output_index].remove(tile.0);
                        continue 'find;
                    }
                }
                chosen_tile = tile;
                break;
            }
            //TODO: check if wavemap boundaries match the chosen tile and if not, look for another one

            //panic!(chosen_tile);
            chosen_tile
        };
        println!();

        entropy_hierarchy.reduce_entropy(collapsing_output_index, least_entropy, 1);

        wave_map.buf[collapsing_output_index].clear();
        wave_map.buf[collapsing_output_index].insert(chosen_tile.0);

        println!(
            "point: {}, tile: {}",
            collapsing_output_index, chosen_tile.0
        );
        println!("chosen tile: {:?}", input_tiles[tiles[chosen_tile]].0);
        println!("wave map before constraints: {:?}", wave_map);
        //entropy_hierarchy.print(&wave_map);
        let tile_count = tiles.0.len();
        // reduce entropy
        let index_2d = wave_map
            .to_2d_index(collapsing_output_index as isize)
            .unwrap();

        constrain(
            &adjacencies,
            &mut wave_map,
            &mut entropy_hierarchy,
            tile_count,
            index_2d,
        );

        println!("wave map after constraints: {:?}", wave_map);

        entropy_hierarchy.cleanup();
        // if result.len() > 400 {
        //     break;
        // }
        result.push(to_image(&input, output_size, &wave_map));
    }

    result.push(to_image(&input, output_size, &wave_map));
    return result;
}

fn constrain(
    adjacencies: &HashMap<V2i, TileMap<BitSet>>,
    wave_map: &mut Grid<BitSet>,
    entropy_hierarchy: &mut EntropyHierarchy,
    tile_count: usize,
    constrainee: V2i,
) -> bool {
    for (&offset, constraint) in adjacencies.iter() {
        let neighbour = constrainee + offset;
        if let Some(inner_index) = wave_map.to_1d_index(neighbour) {
            let reducer = wave_map.get(constrainee).unwrap().iter().fold(
                BitSet::with_capacity(tile_count),
                |mut result, t| {
                    result.union_with(&constraint[TileIndex(t)]);
                    return result;
                },
            );
            if reducer.is_empty() {
                return false;
            }
            let set_to_reduce = wave_map.get_mut(neighbour).unwrap();
            let original = set_to_reduce.len();
            set_to_reduce.intersect_with(&reducer);
            if original != set_to_reduce.len() {
                entropy_hierarchy.reduce_entropy(
                    inner_index as usize,
                    original,
                    set_to_reduce.len(),
                );
                //println!("wave map during constraints: {:?}", wave_map);
                if !constrain(
                    adjacencies,
                    wave_map,
                    entropy_hierarchy,
                    tile_count,
                    neighbour,
                ) {
                    return false;
                }
            }
        }
    }
    return true;
}

// fn get_constraint<T: Copy + Eq + Hash + Debug>(
//     constraints: &Constraints<T>,
//     color: T,
//     maybe_offset: Option<V2i>,
// ) -> &BitSet {
//     match maybe_offset {
//         Some(offset) => constraints.adjacencies.get(&(color, offset)),
//         None => constraints.color_locations.get(&color),
//     }
//     .unwrap()
// }

fn to_image<T: Copy + Eq + Hash + Debug>(
    input: &Grid<T>,
    output_size: Vector2<usize>,
    wave_map: &Grid<BitSet>,
) -> Grid<Result<T, usize>> {
    let mut result = unsafe { Grid::uninitialized_with_capacity(output_size) };
    for (i, set) in wave_map.buf.iter().enumerate() {
        result.buf[i] = (set.len() == 1)
            .then(|| input.buf[set.iter().next().unwrap()])
            .ok_or(set.len());
    }
    return result;
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use cgmath::Array;
    use image::{DynamicImage, Rgb, RgbImage};

    use crate::world_gen::grid::{Grid, V2u};
    use crate::world_gen::wfc::{wfc, SMALL_SQUARE_NEIGHBOURHOOD};

    #[test]
    pub fn test() {
        let timer = Instant::now();
        // let pic = image::open("oliprik.png").unwrap();
        let pic = image::open("samples/SimpleMaze.png").unwrap();
        println!("opened óli prik");
        match pic {
            DynamicImage::ImageRgb8(img) => {
                let size = V2u::from_value(5);
                let master = wfc(
                    Grid::from(&img),
                    Vec::from(SMALL_SQUARE_NEIGHBOURHOOD),
                    size,
                );
                for (i, master) in master.iter().enumerate() {
                    let mut other: Grid<Rgb<u8>> = Grid::new();
                    other.size = master.size;
                    other.buf = master
                        .buf
                        .iter()
                        .map(|pix| match pix {
                            Ok(col) => *col,
                            Err(0) => Rgb([255, 0, 0]),
                            Err(n) => Rgb([0, (*n % 256) as u8, (128 + (*n / 256) * 8) as u8]),
                        })
                        .collect();

                    RgbImage::from(&other)
                        .save(format!("temp/WFC{:03}.png", i))
                        .unwrap();
                    println!("saved result to WFC.png");
                }
            }
            _ => {
                println!("wrong image type!")
            }
        }
        println!("elapsed {:?}", timer.elapsed());
    }
}
