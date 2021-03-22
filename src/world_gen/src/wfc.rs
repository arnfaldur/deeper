use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

use bit_set::BitSet;
use cgmath::{Vector2, Zero};
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
        if index.x >= 0
            && index.y >= 0
            && index.x < self.size.x as isize
            && index.y < self.size.y as isize
        {
            Some(&self[index.map(|e| e as usize)])
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, index: V2i) -> Option<&mut T> {
        if index.x >= 0
            && index.y >= 0
            && index.x < self.size.x as isize
            && index.y < self.size.y as isize
        {
            Some(&mut self[index.map(|e| e as usize)])
        } else {
            None
        }
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

// impl<P> From<ImageBuffer<P, Vec<S>>> for Grid<P>
// where
//     P: Pixel + 'static,
//     P::Subpixel: 'static,
//     C: Deref<Target = [P::Subpixel]>,
// {
impl<P> From<ImageBuffer<P, Vec<P::Subpixel>>> for Grid<P>
where
    P: Pixel + 'static,
{
    fn from(img: ImageBuffer<P, Vec<P::Subpixel>>) -> Self {
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

impl<P> From<Grid<P>> for ImageBuffer<P, Vec<P::Subpixel>>
where
    P: Pixel + 'static,
{
    fn from(grid: Grid<P>) -> Self {
        let mut result = ImageBuffer::new(grid.size.x as u32, grid.size.y as u32);
        for (y, row) in result.rows_mut().enumerate() {
            for (x, pixel) in row.enumerate() {
                *pixel = grid[V2u { y, x }];
            }
        }
        return result;
    }
}

fn wfc<T: Copy + Eq + Hash + Debug>(
    input: Grid<T>,
    neighbourhood: Vec<V2i>,
    output_size: V2u,
) -> Result<Grid<Result<T, i32>>, String> {
    fn to_1d_index(index_2d: V2i, row_size: usize) -> isize {
        index_2d.y * row_size as isize + index_2d.x
    }
    fn to_2d_index(index_1d: isize, row_size: usize) -> V2i {
        V2i::new(index_1d % row_size as isize, index_1d / row_size as isize)
    }
    fn in_bounds(index: V2i, bounds: V2u) -> bool {
        index.x >= 0 && index.y >= 0 && index.x < bounds.x as isize && index.y < bounds.y as isize
    }
    let mut rng = rand::thread_rng();

    let mut wave_map = Grid::new();

    let colors: Vec<T> = input.buf.clone().into_iter().unique().collect();

    let mut color_locations = HashMap::new();
    for color in colors.iter() {
        color_locations.insert(color, BitSet::new());
    }
    let mut locations = Vec::new();
    for (i, color) in input.buf.iter().enumerate() {
        color_locations
            .get_mut(color)
            .and_then(|set| Some(set.insert(i)));
        locations.push(i);
    }

    let mut adjacencies: HashMap<(T, V2i), BitSet> = HashMap::new();
    for &offset in neighbourhood.iter() {
        for &location in locations.iter() {
            if let Some(&color_a) = input.buf.get(location) {
                let index = to_2d_index(location as isize, input.size.x) + offset;
                if in_bounds(index, input.size) {
                    adjacencies
                        .entry((color_a, offset))
                        .or_default()
                        .insert(to_1d_index(index, input.size.x) as usize);
                }
            }
        }
    }

    wave_map.resize(output_size, BitSet::from_iter(locations));

    let mut entropy_hierarchy = EntropyHierarchy {
        hierarchy: BTreeMap::new(),
    };
    for (i, set) in wave_map.buf.iter().enumerate() {
        entropy_hierarchy.add(i, set.len());
    }

    loop {
        // collapse superposition
        let (&_least_entropy, peak) = entropy_hierarchy.get_lowest_entropy();
        let collapse_index = peak.iter().choose(&mut rng).unwrap();
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
        let chosen_color = input.buf[chosen];
        let mister = peak.remove(collapse_index);
        if !mister {
            panic!("getting {} in {:?}", collapse_index, entropy_hierarchy);
        }
        entropy_hierarchy.add(collapse_index, 1);

        wave_map.buf[collapse_index].clear();
        wave_map.buf[collapse_index].insert(chosen);

        // reduce entropy
        for &offset in neighbourhood.iter() {
            if let Some(neighbour_color) =
                input.get(to_2d_index(chosen as isize, input.size.x) + offset)
            {
                if let Some(constraining_set) = color_locations.get(neighbour_color) {
                    let index_2d = to_2d_index(collapse_index as isize, wave_map.size.x) + offset;
                    let index = to_1d_index(index_2d, wave_map.size.x) as usize;
                    if let Some(set_to_reduce) = wave_map.get_mut(index_2d) {
                        let reduced_set: BitSet =
                            set_to_reduce.intersection(constraining_set).collect();
                        entropy_hierarchy.reduce_entropy(
                            index,
                            set_to_reduce.len(),
                            reduced_set.len(),
                        );
                        *set_to_reduce = reduced_set;
                    }
                    for &inner_offset in neighbourhood.iter() {
                        if let Some(inner_constraining_set) =
                            adjacencies.get(&(*neighbour_color, inner_offset))
                        {
                            let inner_index_2d = index_2d + inner_offset;
                            let inner_index = to_1d_index(inner_index_2d, wave_map.size.x) as usize;
                            if inner_index != collapse_index {
                                if let Some(set_to_reduce) = wave_map.get_mut(inner_index_2d) {
                                    let reduced_set: BitSet = set_to_reduce
                                        .intersection(inner_constraining_set)
                                        .collect();

                                    entropy_hierarchy.reduce_entropy(
                                        inner_index,
                                        set_to_reduce.len(),
                                        reduced_set.len(),
                                    );
                                    *set_to_reduce = reduced_set;
                                }
                            }
                        }
                    }
                }
            }
        }

        for (i, set) in wave_map.buf.iter().enumerate() {
            if set.len() > 1 {
                if !entropy_hierarchy
                    .hierarchy
                    .get(&set.len())
                    .unwrap()
                    .contains(i)
                {
                    print!(
                        "malformed entropy hierarchy!: {:?} {} {}",
                        to_2d_index(i as isize, wave_map.size.x),
                        i,
                        set.len()
                    );
                    for e in entropy_hierarchy.hierarchy.iter() {
                        if e.1.contains(set.len()) {
                            println!(" marked as {}", e.0);
                        }
                    }
                }
            }
        }

        entropy_hierarchy.cleanup();

        // find points of lowest entropy
        if entropy_hierarchy.is_converged() {
            break;
        }
    }

    let mut result = Grid::with_capacity(output_size);
    unsafe {
        result.set_len(output_size);
    }
    for (i, set) in wave_map.buf.iter().enumerate() {
        // if set.len() > 1 {
        //     return Err("WFC failed: entropy is more than 1 somewhere".to_string());
        // } else if set.len() < 1 {
        //     return Err("WFC failed: entropy is less than 1 somewhere".to_string());
        // } else
        {
            result.buf[i] = if set.len() > 1 {
                Err(2)
            } else if set.len() < 1 {
                Err(0)
            } else {
                Ok(input.buf[set.iter().next().unwrap()])
            };
        }
    }
    return Ok(result);
}

pub fn test() {
    let oli = image::open("samples/Flowers.png").unwrap();
    println!("opened óli prik");
    match oli {
        DynamicImage::ImageRgb8(img) => {
            let size = V2u::new(64, 64);
            let master = wfc(img.into(), Vec::from(SQUARE_NEIGHBOURHOOD), size).unwrap();
            let mut other: Grid<Rgb<u8>> = Grid::new();
            other.size = master.size;
            other.buf = master
                .buf
                .iter()
                .map(|pix| match pix {
                    Ok(col) => *col,
                    Err(0) => Rgb([255, 0, 0]),
                    Err(2) => Rgb([0, 255, 0]),
                    Err(_) => Rgb([0, 0, 255]),
                })
                .collect();

            RgbImage::from(other).save("WFC.png").unwrap();
            println!("saved result to WFC.png");
        }
        _ => {
            println!("wrong image type!")
        }
    }
}
