use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};
use std::time::Instant;

use bit_set::BitSet;
use cgmath::{Array, Vector2, Zero};
use image::{DynamicImage, ImageBuffer, Pixel, Rgb, RgbImage};
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

// impl<P> From<ImageBuffer<P, Vec<S>>> for Grid<P>
// where
//     P: Pixel + 'static,
//     P::Subpixel: 'static,
//     C: Deref<Target = [P::Subpixel]>,
// {
impl<P> From<&ImageBuffer<P, Vec<P::Subpixel>>> for Grid<P>
where
    P: Pixel + 'static,
{
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

impl<P> From<&Grid<P>> for ImageBuffer<P, Vec<P::Subpixel>>
where
    P: Pixel + 'static,
{
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

fn to_1d_index(index_2d: V2i, row_size: usize) -> isize {
    index_2d.y * row_size as isize + index_2d.x
}

fn to_2d_index(index_1d: isize, row_size: usize) -> V2i {
    V2i::new(index_1d % row_size as isize, index_1d / row_size as isize)
}

fn in_bounds(index: V2i, bounds: V2u) -> bool {
    index.x >= 0 && index.y >= 0 && index.x < bounds.x as isize && index.y < bounds.y as isize
}

fn wfc<T: Copy + Eq + Hash + Debug>(
    input: Grid<T>,
    neighbourhood: Vec<V2i>,
    output_size: V2u,
) -> Vec<Grid<Result<T, usize>>> {
    let mut rng = rand::thread_rng();

    let mut wave_map = Grid::new();

    let mut color_locations: HashMap<&T, BitSet> = HashMap::new();
    for (i, color) in input.buf.iter().enumerate() {
        color_locations.entry(color).or_default().insert(i);
    }
    let locations: Vec<usize> = Vec::from_iter(0..input.buf.len());

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

    let thing = BitSet::from_iter(locations);
    wave_map.resize(output_size, thing.clone());

    let mut entropy_hierarchy = EntropyHierarchy::new();
    for (i, set) in wave_map.buf.iter().enumerate() {
        entropy_hierarchy.add(i, set.len());
    }

    let mut result = Vec::new();
    loop {
        // collapse superposition
        let (&least_entropy, peak) = entropy_hierarchy.get_lowest_entropy();
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

        entropy_hierarchy.reduce_entropy(collapse_index, least_entropy, 1);

        wave_map.buf[collapse_index].clear();
        wave_map.buf[collapse_index].insert(chosen);

        // reduce entropy
        for &offset in neighbourhood.iter() {
            if let Some(neighbour_color) =
                input.get(to_2d_index(chosen as isize, input.size.x) + offset)
            {
                if let Some(constraining_set) = color_locations.get(neighbour_color) {
                    let index_2d = to_2d_index(collapse_index as isize, wave_map.size.x) + offset;
                    smoething(
                        &mut wave_map,
                        &mut entropy_hierarchy,
                        constraining_set,
                        index_2d,
                    );
                    for &inner_offset in neighbourhood.iter() {
                        if let Some(inner_constraining_set) =
                            adjacencies.get(&(*neighbour_color, inner_offset))
                        {
                            let inner_index_2d = index_2d + inner_offset;
                            smoething(
                                &mut wave_map,
                                &mut entropy_hierarchy,
                                inner_constraining_set,
                                inner_index_2d,
                            )
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

    result.push(to_image(&input, output_size, &wave_map));
    return result;
}

fn smoething(
    wave_map: &mut Grid<BitSet>,
    entropy_hierarchy: &mut EntropyHierarchy,
    constraining_set: &BitSet,
    index_2d: V2i,
) {
    let index = to_1d_index(index_2d, wave_map.size.x) as usize;
    if let Some(set_to_reduce) = wave_map.get_mut(index_2d) {
        let original = set_to_reduce.len();
        set_to_reduce.intersect_with(constraining_set);
        entropy_hierarchy.reduce_entropy(index, original, set_to_reduce.len());
    }
}

fn to_image<T: Copy + Eq + Hash + Debug>(
    input: &Grid<T>,
    output_size: Vector2<usize>,
    wave_map: &Grid<BitSet>,
) -> Grid<Result<T, usize>> {
    let mut result = unsafe { Grid::uninitialized_with_capacity(output_size) };
    for (i, set) in wave_map.buf.iter().enumerate() {
        // if set.len() > 1 {
        //     return Err("WFC failed: entropy is more than 1 somewhere".to_string());
        // } else if set.len() < 1 {
        //     return Err("WFC failed: entropy is less than 1 somewhere".to_string());
        // } else
        {
            result.buf[i] = if set.len() != 1 {
                Err(set.len())
            } else {
                Ok(input.buf[set.iter().next().unwrap()])
            };
        }
    }
    return result;
}

pub fn test() {
    let timer = Instant::now();
    let oli = image::open("oliprik.png").unwrap();
    println!("opened óli prik");
    match oli {
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
