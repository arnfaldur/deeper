use std::cmp::min;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

use bit_set::BitSet;
use cgmath::{Vector2, Zero};
use image::{DynamicImage, Rgb, RgbImage};
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
    let mut rng = rand::thread_rng();

    let mut wave_map = Grid::new();

    let mut thing = HashSet::new();
    for color in input.buf.clone().into_iter() {
        thing.insert(color);
    }
    let colors: Vec<T> = thing.into_iter().collect();

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
    println!("colors located");

    for loc in color_locations.iter() {
        println!("{:?} -> {}", loc.0, loc.1.len());
    }

    let mut adjacencies: HashMap<(T, V2i), BitSet> = HashMap::new();
    for &offset in neighbourhood.iter() {
        for &location in locations.iter() {
            if let Some(&color_a) = input.buf.get(location) {
                let index = to_2d_index(location as isize, input.size.x) + offset;
                if index.x >= 0
                    && index.y >= 0
                    && index.x < input.size.x as isize
                    && index.y < input.size.y as isize
                {
                    adjacencies
                        .entry((color_a, offset))
                        .or_default()
                        .insert(to_1d_index(index, input.size.x) as usize);
                }
            }
        }
    }
    println!("ting realized");

    for boi in adjacencies.iter() {
        println!("{:?} -> {}", boi.0, boi.1.len());
    }

    wave_map.resize(output_size, BitSet::from_iter(locations));

    let mut entropy_hierarchy: BTreeMap<usize, HashSet<usize>> = BTreeMap::new();
    for (i, set) in wave_map.buf.iter().enumerate() {
        entropy_hierarchy.entry(set.len()).or_default().insert(i);
    }

    'collapser: loop {
        // collapse superposition
        let mut biib = entropy_hierarchy.first_entry().unwrap();
        let collapse_index = *biib.get().iter().choose(&mut rng).unwrap();
        println!("collapsing: {:?}", collapse_index);
        let chosen = wave_map.buf[collapse_index]
            .iter()
            .choose(&mut rng)
            .unwrap();
        let chosen_color = input.buf[chosen];
        biib.get_mut().remove(&collapse_index);
        wave_map.buf[collapse_index].clear();
        wave_map.buf[collapse_index].insert(chosen);

        // reduce entropy
        for &offset in neighbourhood.iter() {
            if let Some(constraining_set) = adjacencies.get(&(chosen_color, offset)) {
                let index =
                    (collapse_index as isize + to_1d_index(offset, wave_map.size.x)) as usize;
                if let Some(set_to_reduce) = wave_map.buf.get_mut(index) {
                    // println!(
                    //     "{:?} ro {} co {}",
                    //     offset,
                    //     set_to_reduce.len(),
                    //     constraining_set.len()
                    // );
                    let boi = entropy_hierarchy
                        .get_mut(&set_to_reduce.len())
                        .map(|e| e.remove(&index));

                    set_to_reduce.intersect_with(constraining_set);

                    if set_to_reduce.len() > 1 {
                        entropy_hierarchy
                            .entry(set_to_reduce.len())
                            .or_default()
                            .insert(index);
                    }
                }
            }
        }

        for (i, set) in wave_map.buf.iter().enumerate() {
            if set.len() > 1 {
                if !entropy_hierarchy.get(&set.len()).unwrap().contains(&i) {
                    println!(
                        "{:?} {} {}",
                        to_2d_index(i as isize, wave_map.size.x),
                        i,
                        set.len()
                    );
                }
            }
        }

        if entropy_hierarchy.remove(&0).is_some() {
            println!("removing 0");
        }
        if entropy_hierarchy.remove(&1).is_some() {
            println!("removing 1");
        }
        while let Some(true) = entropy_hierarchy.first_entry().map(|e| e.get().is_empty()) {
            entropy_hierarchy.first_entry().unwrap().remove();
        }

        // find points of lowest entropy
        if entropy_hierarchy.is_empty() {
            break;
        }
    }

    println!("done collapsing");

    let mut result = Grid::with_capacity(output_size);
    unsafe {
        result.set_len(output_size);
    }
    println!("did an unsafe thing");
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
    println!("Made the result");
    return Ok(result);
}

pub fn test() {
    let oli = image::open("oliprik.png").unwrap();
    println!("opened óli prik");
    match oli {
        DynamicImage::ImageRgb8(img) => {
            let mut ingrid = Grid::new();
            ingrid.resize(
                V2u {
                    y: img.height() as usize,
                    x: img.width() as usize,
                },
                Rgb([0, 0, 0]),
            );
            for (y, row) in img.rows().enumerate() {
                for (x, pixel) in row.enumerate() {
                    ingrid[V2u { y, x }] = *pixel;
                }
            }
            println!("óli is grid");
            let size = V2u::new(128, 128);
            let master = wfc(ingrid, Vec::from(SQUARE_NEIGHBOURHOOD), size).unwrap();
            println!("óli has been collapsed");
            let mut output: RgbImage = RgbImage::new(size.x as u32, size.y as u32);
            for (y, row) in output.rows_mut().enumerate() {
                for (x, pixel) in row.enumerate() {
                    let boi = master[V2u { y, x }];
                    *pixel = match boi {
                        Ok(col) => col,
                        Err(0) => Rgb([255, 0, 0]),
                        Err(2) => Rgb([0, 255, 0]),
                        Err(_) => Rgb([0, 0, 255]),
                    }
                }
            }
            println!("óli is imaged again");

            output.save("WFC.png").unwrap();
            println!("óli is safe");
        }
        _ => {
            println!("wrong image type!")
        }
    }
}
