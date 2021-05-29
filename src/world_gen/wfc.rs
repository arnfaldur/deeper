use std::cmp::{max, min};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::iter::{FromIterator, Map};
use std::ops::{Index, IndexMut, Range};

use bit_set::BitSet;
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
#[derive(Clone, Debug)]
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
    pub fn print<F>(&self, neighbourhood: &[V2i], printer: F)
    where
        F: Fn(Option<T>) -> String,
        T: Clone,
    {
        let (ty, tx, by, bx) = neighbourhood.iter().fold(
            (isize::MIN, isize::MIN, isize::MAX, isize::MAX),
            |(ty, tx, by, bx), a| {
                return (max(a.y, ty), max(a.x, tx), min(a.y, by), min(a.x, bx));
            },
        );

        for y in by..=ty {
            for x in bx..=tx {
                let v = V2i::new(x, y);
                let mut i = usize::MAX;
                for (j, &off) in neighbourhood.iter().enumerate() {
                    if off == v {
                        i = j;
                        break;
                    }
                }
                print!("{}", printer(self.0[i].clone()));
            }
            println!();
        }
        //println!("asdf: {:?}", limits);
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

#[allow(dead_code)]
pub enum Orientation {
    Original,
    Clockwise90,
    Clockwise180,
    Clockwise270,
}

pub fn wfc<T: Copy + Eq + Hash + Debug>(
    input: Grid<T>,
    neighbourhood: &[V2i],
    output_size: V2u,
    orientations: &[Orientation],
) -> Result<Vec<Grid<Result<T, usize>>>, String> {
    let mut rng = rand::prelude::StdRng::seed_from_u64(1337);

    let printing = false;

    let inverse_neighbourhood: HashMap<V2i, usize> = neighbourhood
        .iter()
        .enumerate()
        .map(|(i, off)| (*off, i))
        .collect();
    if printing {
        println!("{:?}", inverse_neighbourhood);
    }

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
            .unique_by(|(_, &tile_hash)| tile_hash)
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
    if printing {
        println!("inputs to tiles {:?}", input_to_tiles);
    }
    // the inverse of the above
    let tiles_to_inputs: TileMap<Vec<InputIndex>> = input_to_tiles.0.iter().enumerate().fold(
        TileMap(tiles.0.iter().map(|_| Vec::new()).collect()),
        |mut result, (i, &tile_index)| {
            result[tile_index].push(InputIndex(i));
            return result;
        },
    );
    if printing {
        println!("tiles to inputs {:?}", tiles_to_inputs);
    }
    let mut adjacencies: HashMap<V2i, TileMap<BitSet>> = neighbourhood
        .iter()
        .map(|e| *e)
        .chain(neighbourhood.iter().map(|&off| -off))
        .filter(|offset| *offset != V2i::new(0, 0))
        .fold(HashMap::new(), |mut result, offset| {
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
    if printing {
        println!("adjacencies v1 {:?}", adjacencies);
    }
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
    if printing {
        println!("adjacencies v2 {:?}", adjacencies);
    }

    // let boi: Vec<V2i> = HashSet::from_iter(neighbourhood.iter()).intersection(&HashSet::from_iter(
    //     neighbourhood.iter().map(|of| of + offset),
    // )).collect();
    let mut wave_map = Grid::new();
    wave_map.resize(output_size, BitSet::from_iter(0..tiles.0.len()));
    // TODO: consider using this to make reversable steps
    let mut wave_map_backup = wave_map.clone();
    if printing {
        for i in tiles.range() {
            println!("tile {}", i.0);

            let white = input_tiles[InputIndex(0)].0[0].clone();
            let black = input_tiles[InputIndex(0)].0[3].clone();

            let boii = move |thing| {
                String::from(if thing == white {
                    "W"
                } else if thing == black {
                    "b"
                } else {
                    "."
                })
            };

            input_tiles[tiles[i]].print(&neighbourhood, boii);
            // println!("{:?}", input_tiles[tiles[i]]);
        }
        println!("{:?}, ", neighbourhood);
        for (offset, _) in adjacencies
            .iter()
            .sorted_by(|(a, _), (b, _)| a.y.cmp(&b.y).then_with(|| a.x.cmp(&b.x)))
        {
            print!("{:?}, ", offset);
        }
        println!();
        for i in tiles.range() {
            print!("tile: {} -> ", i.0);
            for (_, bit_set) in adjacencies
                .iter()
                .sorted_by(|(a, _), (b, _)| a.y.cmp(&b.y).then_with(|| a.x.cmp(&b.x)))
            {
                print!("{:?}, ", bit_set[i]);
            }
            println!();
        }
    }

    let mut entropy_hierarchy = EntropyHierarchy::new();
    for (i, set) in wave_map.buf.iter().enumerate() {
        entropy_hierarchy.add(i, set.len());
    }
    let mut entropy_hierarchy_backup = entropy_hierarchy.clone();
    let tile_count = tiles.0.len();

    let possible = constrain(
        &adjacencies,
        &mut wave_map,
        &mut entropy_hierarchy,
        tile_count,
        V2i::new(0, 0),
    );
    if printing {
        println!("wave map start: {:?}", wave_map);
    }
    let mut result = Vec::new();

    if !possible {
        return Err(String::from(
            "input can't be used to generate a valid output",
        ));
    }

    while !entropy_hierarchy.is_converged() {
        if printing {
            println!("---------- iteration ----------");
        }
        // collapse superposition
        let (&least_entropy, bottom) = entropy_hierarchy.get_lowest_entropy();
        let collapsing_output_index = bottom.iter().choose(&mut rng).unwrap();
        // println!(
        //     "collapsing: {:?} {:?} with entropy {}",
        //     to_2d_index(collapse_index as isize, wave_map.size.x),
        //     collapse_index,
        //     least_entropy
        // );
        if printing {
            println!("point: {}", collapsing_output_index);
            print!("choosing tile: ");
        }
        let chosen_tile = wave_map.buf[collapsing_output_index]
            .iter()
            .choose(&mut rng)
            .map(|x| TileIndex(x))
            .unwrap();
        //TODO: check if wavemap boundaries match the chosen tile and if not, look for another one
        if printing {
            println!();
        }
        entropy_hierarchy.reduce_entropy(collapsing_output_index, least_entropy, 1);
        wave_map.buf[collapsing_output_index].clear();
        wave_map.buf[collapsing_output_index].insert(chosen_tile.0);

        if printing {
            println!("tile: {}", chosen_tile.0);
            println!("chosen tile: {:?}", input_tiles[tiles[chosen_tile]].0);
            println!("wave map before constraints: {:?}", wave_map);
            //entropy_hierarchy.print(&wave_map);
        }

        // reduce entropy
        let index_2d = wave_map
            .to_2d_index(collapsing_output_index as isize)
            .unwrap();

        if !constrain(
            &adjacencies,
            &mut wave_map,
            &mut entropy_hierarchy,
            tile_count,
            index_2d,
        ) {
            wave_map_backup.buf[collapsing_output_index].remove(chosen_tile.0);
            entropy_hierarchy_backup.reduce_entropy(
                collapsing_output_index,
                least_entropy,
                least_entropy - 1,
            );
            wave_map = wave_map_backup.clone();
            entropy_hierarchy = entropy_hierarchy_backup.clone();
        } else {
            if printing {
                println!("wave map after constraints: {:?}", wave_map);
            }
            entropy_hierarchy.cleanup();

            wave_map_backup = wave_map.clone();
            entropy_hierarchy_backup = entropy_hierarchy.clone();
            // if result.len() > 400 {
            //     break;
            // }
        }
    }

    let mut image = unsafe { Grid::uninitialized_with_capacity(output_size) };
    for (i, set) in wave_map.buf.iter().enumerate() {
        image.buf[i] = (set.len() == 1)
            .then(|| input.buf[tiles_to_inputs[TileIndex(set.iter().next().unwrap())][0].0])
            .ok_or(set.len());
    }
    result.push(image);
    //result.push(to_image(&input, output_size, &wave_map));
    return Ok(result);
}

fn constrain(
    adjacencies: &HashMap<V2i, TileMap<BitSet>>,
    wave_map: &mut Grid<BitSet>,
    entropy_hierarchy: &mut EntropyHierarchy,
    tile_count: usize,
    constrainee: V2i,
) -> bool {
    let mut stack = Vec::new();
    stack.push(constrainee);
    while let Some(constrainee) = stack.pop() {
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
                    stack.push(neighbour);
                }
            }
        }
    }
    return true;
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use cgmath::Array;
    use image::{Rgb, RgbImage};

    use crate::world_gen::grid::{Grid, V2u};
    #[allow(unused_imports)]
    use crate::world_gen::wfc::{
        wfc, Orientation, SMALL_SQUARE_NEIGHBOURHOOD, SQUARE_NEIGHBOURHOOD,
    };

    #[test]
    pub fn test() {
        let timer = Instant::now();
        // let pic = image::open("oliprik.png").unwrap();
        let pic = image::open("samples/Cats.png").unwrap();
        println!("opened óli prik");
        let size = V2u::from_value(64);
        let master = wfc(
            Grid::from(&pic.into_rgb8()),
            &SQUARE_NEIGHBOURHOOD,
            size,
            &[Orientation::Original, Orientation::Clockwise180],
        );
        match master {
            Result::Ok(res) => {
                for (i, master) in res.iter().enumerate() {
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
            Result::Err(err) => println!("Error! {}", err),
        }
        println!("elapsed {:?}", timer.elapsed());
    }
}
