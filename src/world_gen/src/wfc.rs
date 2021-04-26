use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};
use std::time::Instant;

use bit_set::BitSet;
use cgmath::{Array, Vector2, Zero};
use image::{DynamicImage, ImageBuffer, Pixel, Rgb, RgbImage};
use itertools::Itertools;
use rand::prelude::*;

type V2u = Vector2<usize>;
type V2i = Vector2<isize>;

const SQUARE_NEIGHBOURHOOD: [V2i; 8] = [
    V2i::new(-1, 0),
    V2i::new(-1, 1),
    V2i::new(0, 1),
    V2i::new(1, 1),
    V2i::new(1, 0),
    V2i::new(1, -1),
    V2i::new(0, -1),
    V2i::new(-1, -1),
];
#[allow(dead_code)]
const CROSS_NEIGHBOURHOOD: [V2i; 4] = [
    V2i::new(-1, 0),
    V2i::new(0, 1),
    V2i::new(1, 0),
    V2i::new(0, -1),
];

#[derive(Clone)]
struct Grid<T> {
    size: V2u,
    buf: Vec<T>,
}

impl<T> Grid<T> {
    pub fn new() -> Self {
        Self {
            size: V2u::zero(),
            buf: Vec::new(),
        }
    }
    pub fn with_capacity(capacity: V2u) -> Self {
        Self {
            size: V2u::zero(),
            buf: Vec::with_capacity(capacity.y * capacity.x),
        }
    }
    pub unsafe fn uninitialized_with_capacity(capacity: V2u) -> Self {
        let mut result = Self::with_capacity(capacity);
        result.set_len(capacity);
        return result;
    }
    pub fn resize(&mut self, size: V2u, value: T)
    where
        T: Clone,
    {
        self.size = size;
        self.buf.resize(size.x * size.y, value);
    }

    pub unsafe fn set_len(&mut self, size: V2u) {
        self.size = size;
        self.buf.set_len(size.x * size.y);
    }
    pub fn get(&self, index: V2i) -> Option<&T> {
        self.in_bounds(index)
            .then(move || &self[index.map(|e| e as usize)])
    }
    pub fn get_mut(&mut self, index: V2i) -> Option<&mut T> {
        self.in_bounds(index)
            .then(move || &mut self[index.map(|e| e as usize)])
    }

    fn in_bounds(&self, index: V2i) -> bool {
        index.x >= 0
            && index.y >= 0
            && index.x < self.size.x as isize
            && index.y < self.size.y as isize
    }
    fn to_1d_index(&self, index_2d: V2i) -> Option<isize> {
        return self
            .in_bounds(index_2d)
            .then(|| index_2d.y * self.size.x as isize + index_2d.x);
    }
    fn to_2d_index(&self, index_1d: isize) -> Option<V2i> {
        let result = V2i::new(
            index_1d % self.size.x as isize,
            index_1d / self.size.x as isize,
        );
        return self.in_bounds(result).then(|| result);
    }
}

impl<T> Index<V2u> for Grid<T> {
    type Output = T;
    fn index(&self, index: V2u) -> &Self::Output { &self.buf[index.y * self.size.x + index.x] }
}
impl<T> Index<&V2u> for Grid<T> {
    type Output = T;
    fn index(&self, index: &V2u) -> &Self::Output { &self.buf[index.y * self.size.x + index.x] }
}
impl<T> IndexMut<V2u> for Grid<T> {
    fn index_mut(&mut self, index: V2u) -> &mut Self::Output {
        &mut self.buf[index.y * self.size.x + index.x]
    }
}
impl<T> IndexMut<&V2u> for Grid<T> {
    fn index_mut(&mut self, index: &V2u) -> &mut Self::Output {
        &mut self.buf[index.y * self.size.x + index.x]
    }
}

#[derive(Debug)]
struct EntropyHierarchy {
    hierarchy: BTreeMap<usize, BitSet>,
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
}

impl<P: Pixel + 'static> From<&ImageBuffer<P, Vec<P::Subpixel>>> for Grid<P> {
    fn from(img: &ImageBuffer<P, Vec<P::Subpixel>>) -> Self {
        let image_size = V2u {
            y: img.height() as usize,
            x: img.width() as usize,
        };
        let mut result = unsafe { Grid::uninitialized_with_capacity(image_size) };
        for (y, row) in img.rows().enumerate() {
            for (x, pixel) in row.enumerate() {
                result[V2u { y, x }] = *pixel;
            }
        }
        return result;
    }
}

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

struct Constraints<T> {
    color_locations: HashMap<T, BitSet>,
    adjacencies: HashMap<(T, V2i), BitSet>,
}

#[allow(dead_code)]
fn wfc_old<T: Copy + Eq + Hash + Debug>(
    input: Grid<T>,
    neighbourhood: Vec<V2i>,
    output_size: V2u,
) -> Vec<Grid<Result<T, usize>>> {
    let mut rng = rand::prelude::StdRng::seed_from_u64(1337);

    let constraints = {
        let mut constraints = Constraints {
            color_locations: HashMap::new(),
            adjacencies: HashMap::new(),
        };
        for (i, &color) in input.buf.iter().enumerate() {
            constraints
                .color_locations
                .entry(color)
                .or_default()
                .insert(i);
        }
        for &offset in neighbourhood.iter() {
            for (location, &color_a) in input.buf.iter().enumerate() {
                let index = input.to_2d_index(location as isize).unwrap() + offset;
                if let Some(spot) = input.to_1d_index(index) {
                    constraints
                        .adjacencies
                        .entry((color_a, offset))
                        .or_default()
                        .insert(spot as usize);
                }
            }
        }
        constraints
    };

    let mut wave_map = Grid::new();
    wave_map.resize(output_size, BitSet::from_iter(0..input.buf.len()));

    let mut entropy_hierarchy = EntropyHierarchy::new();
    for (i, set) in wave_map.buf.iter().enumerate() {
        entropy_hierarchy.add(i, set.len());
    }

    let mut result = Vec::new();
    while !entropy_hierarchy.is_converged() {
        // collapse superposition
        let (&least_entropy, bottom) = entropy_hierarchy.get_lowest_entropy();
        let collapse_index = bottom.iter().choose(&mut rng).unwrap();
        // println!(
        //     "collapsing: {:?} {:?} with entropy {}",
        //     to_2d_index(collapse_index as isize, wave_map.size.x),
        //     collapse_index,
        //     least_entropy
        // );
        let chosen = wave_map.buf[collapse_index]
            .iter()
            .choose(&mut rng)
            .unwrap();

        entropy_hierarchy.reduce_entropy(collapse_index, least_entropy, 1);

        wave_map.buf[collapse_index].clear();
        wave_map.buf[collapse_index].insert(chosen);

        // reduce entropy
        let wave_map_argument = &mut wave_map;
        let entropy_hierarchy_argument = &mut entropy_hierarchy;
        let input_argument = &input;
        let neighbourhood_argument = &neighbourhood;
        let constraints_argument = &constraints;
        for &offset in neighbourhood_argument.iter() {
            if let Some(&neighbour_color) =
                input_argument.get(input_argument.to_2d_index(chosen as isize).unwrap() + offset)
            {
                let index_2d = wave_map_argument
                    .to_2d_index(collapse_index as isize)
                    .unwrap()
                    + offset;
                let constraining_set = get_constraint(&constraints_argument, neighbour_color, None);
                if let Some(index) = wave_map_argument.to_1d_index(index_2d) {
                    let set_to_reduce = wave_map_argument.get_mut(index_2d).unwrap();
                    let original = set_to_reduce.len();
                    set_to_reduce.intersect_with(constraining_set);
                    if original != set_to_reduce.len() {
                        entropy_hierarchy_argument.reduce_entropy(
                            index as usize,
                            original,
                            set_to_reduce.len(),
                        );
                    }
                }
                for &offset in neighbourhood_argument.iter() {
                    let index_2d = index_2d + offset;
                    let constraining_set =
                        get_constraint(constraints_argument, neighbour_color, Some(offset));
                    if let Some(index) = wave_map_argument.to_1d_index(index_2d) {
                        let set_to_reduce = wave_map_argument.get_mut(index_2d).unwrap();
                        let original = set_to_reduce.len();
                        set_to_reduce.intersect_with(constraining_set);
                        if original != set_to_reduce.len() {
                            entropy_hierarchy_argument.reduce_entropy(
                                index as usize,
                                original,
                                set_to_reduce.len(),
                            );
                            // if let Some(&neighbour_color) =
                            //     input.get(input.to_2d_index(chosen as isize).unwrap() + offset)
                            // {
                            //     recurse(
                            //         wave_map,
                            //         entropy_hierarchy,
                            //         input,
                            //         neighbourhood,
                            //         constraints,
                            //         neighbour_color,
                            //         index_2d,
                            //     );
                            // }
                        }
                    }
                }
            }
        }
        entropy_hierarchy.cleanup();
    }

    result.push(to_image(&input, output_size, &wave_map));
    return result;
}

fn wfc<T: Copy + Eq + Hash + Debug>(
    input: Grid<T>,
    neighbourhood: Vec<V2i>,
    output_size: V2u,
) -> Vec<Grid<Result<T, usize>>> {
    let mut rng = rand::prelude::StdRng::seed_from_u64(1337);

    type Tile<T> = Vec<Option<T>>;
    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
    struct TileHash(u64); // a hash of a Tile<T>
    #[derive(Debug)]
    struct InputMap<T>(Vec<T>); // a set of things mapped to the input grid
    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
    struct InputIndex(usize); // an index into the InputMap
    #[derive(Debug)]
    struct TileMap<T>(Vec<T>); // a set of things mapped to each unique tile
    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
    struct TileIndex(usize); // an index into the TileMap

    // a tile for each point in the input
    let input_tiles: InputMap<Tile<T>> = InputMap(
        (0..input.buf.len())
            .map(|i| {
                let index_2d = input.to_2d_index(i as isize).unwrap();
                neighbourhood
                    .iter()
                    .map(|&offset| input.get(index_2d + offset).map(|c| *c))
                    .collect()
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
            .map(|(tile_id, input_id)| (input_tile_hashes.0[input_id.0], TileIndex(tile_id))),
    );
    // a mapping from input indices to indexes of a TileMap
    let input_to_tiles: InputMap<TileIndex> = InputMap(
        input_tile_hashes
            .0
            .iter()
            .map(|hash| hash_to_tile_index[hash])
            .collect(),
    );
    // the inverse of the above
    let tile_to_inputs: TileMap<Vec<InputIndex>> = input_to_tiles.0.iter().enumerate().fold(
        TileMap(tiles.0.iter().map(|_| Vec::new()).collect()),
        |mut result, (i, &TileIndex(tile_index))| {
            result.0[tile_index].push(InputIndex(i));
            return result;
        },
    );
    let adjacencies: HashMap<V2i, TileMap<BitSet>> =
        neighbourhood
            .iter()
            .fold(HashMap::new(), |mut result, &offset| {
                result
                    .entry(offset)
                    .or_insert(TileMap(Vec::new()))
                    .0
                    .extend((0..tiles.0.len()).map(|tile_index| {
                        BitSet::from_iter(
                            tile_to_inputs.0.get(tile_index).unwrap().iter().filter_map(
                                |&InputIndex(input_id)| {
                                    input
                                        .to_1d_index(
                                            input.to_2d_index(input_id as isize).unwrap() + offset,
                                        )
                                        .map(|input_neighbour| {
                                            input_to_tiles.0[input_neighbour as usize].0
                                        })
                                },
                            ),
                        )
                    }));
                return result;
            });
    println!("oh no! {:?}", adjacencies);
    let mut wave_map = Grid::new();
    wave_map.resize(output_size, BitSet::from_iter(0..tiles.0.len()));

    let mut entropy_hierarchy = EntropyHierarchy::new();
    for (i, set) in wave_map.buf.iter().enumerate() {
        entropy_hierarchy.add(i, set.len());
    }

    let mut result = Vec::new();
    while !entropy_hierarchy.is_converged() {
        // collapse superposition
        let (&least_entropy, bottom) = entropy_hierarchy.get_lowest_entropy();
        let collapsing_output_index = bottom.iter().choose(&mut rng).unwrap();
        // println!(
        //     "collapsing: {:?} {:?} with entropy {}",
        //     to_2d_index(collapse_index as isize, wave_map.size.x),
        //     collapse_index,
        //     least_entropy
        // );
        let chosen_tile = {
            let result = wave_map.buf[collapsing_output_index]
                .iter()
                .choose(&mut rng)
                .unwrap();
            //TODO: check if wavemap boundrys match the chosen tile and if not, look for another one
            result
        };

        entropy_hierarchy.reduce_entropy(collapsing_output_index, least_entropy, 1);

        wave_map.buf[collapsing_output_index].clear();
        wave_map.buf[collapsing_output_index].insert(chosen_tile);

        // reduce entropy
        for (&offset, constraint) in adjacencies.iter() {
            let index_2d = wave_map
                .to_2d_index(collapsing_output_index as isize)
                .unwrap()
                + offset;
            if let Some(index) = wave_map.to_1d_index(index_2d) {
                let set_to_reduce = wave_map.get_mut(index_2d).unwrap();
                let original = set_to_reduce.len();
                set_to_reduce.intersect_with(&constraint.0[chosen_tile]);
                if original != set_to_reduce.len() {
                    entropy_hierarchy.reduce_entropy(index as usize, original, set_to_reduce.len());

                    // for (&inner_offset, inner_constraint) in adjacencies.iter() {
                    //     let inner_index_2d = index_2d + inner_offset;
                    //     if let Some(inner_index) = wave_map.to_1d_index(inner_index_2d) {
                    //         let reducer = wave_map.get(index_2d).unwrap().iter().fold(
                    //             BitSet::with_capacity(tiles.0.len()),
                    //             |mut result, t| {
                    //                 result.union_with(&inner_constraint.0[t]);
                    //                 return result;
                    //             },
                    //         );
                    //         let inner_set_to_reduce = wave_map.get_mut(inner_index_2d).unwrap();
                    //         let inner_original = inner_set_to_reduce.len();
                    //         inner_set_to_reduce.intersect_with(&reducer);
                    //         if inner_original != inner_set_to_reduce.len() {
                    //             entropy_hierarchy.reduce_entropy(
                    //                 inner_index as usize,
                    //                 inner_original,
                    //                 inner_set_to_reduce.len(),
                    //             );
                    //         }
                    //     }
                    // }
                }
            }
        }
        entropy_hierarchy.cleanup();
        // if result.len() > 400 {
        //     break;
        // }
        // result.push(to_image(&input, output_size, &wave_map));
    }

    result.push(to_image(&input, output_size, &wave_map));
    return result;
}

fn get_constraint<T: Copy + Eq + Hash + Debug>(
    constraints: &Constraints<T>,
    color: T,
    maybe_offset: Option<V2i>,
) -> &BitSet {
    match maybe_offset {
        Some(offset) => constraints.adjacencies.get(&(color, offset)),
        None => constraints.color_locations.get(&color),
    }
    .unwrap()
}

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

pub fn test() {
    let timer = Instant::now();
    // let pic = image::open("oliprik.png").unwrap();
    let pic = image::open("samples/Flowers.png").unwrap();
    println!("opened óli prik");
    match pic {
        DynamicImage::ImageRgb8(img) => {
            let size = V2u::from_value(256);
            let master = wfc(Grid::from(&img), Vec::from(SQUARE_NEIGHBOURHOOD), size);
            for (i, master) in master.iter().enumerate() {
                let mut other: Grid<Rgb<u8>> = Grid::new();
                other.size = master.size;
                other.buf = master
                    .buf
                    .iter()
                    .map(|pix| match pix {
                        Ok(col) => *col,
                        Err(0) => Rgb([255, 0, 0]),
                        Err(n) => Rgb([0, (*n % 256) as u8, (60 + (*n / 256) * 10) as u8]),
                    })
                    .collect();

                RgbImage::from(&other)
                    .save(format!("temp/WFC{}.png", i))
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
