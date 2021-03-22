use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

use cgmath::{Vector2, Zero};
use rand::prelude::*;

type V2u = Vector2<usize>;
type V2i = Vector2<isize>;

type Color = usize;

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

fn wfc(input: Grid<u8>, neighbourhood: Vec<V2i>, output_size: V2u) -> Result<Grid<u8>, String> {
    let mut rng = rand::thread_rng();

    let mut wave_map = Grid::new();

    let mut thing = HashSet::new();
    for e in input.buf.clone().into_iter() {
        thing.insert(e);
    }
    let colors: Vec<u8> = thing.into_iter().collect();

    let mut color_locations = HashMap::new();
    for color in colors.iter() {
        color_locations.insert(color, HashSet::new());
    }

    let mut locations = Vec::new();
    for (i, color) in input.buf.iter().enumerate() {
        let spot = V2u::new(i % input.size.x, i / input.size.x);
        color_locations
            .get_mut(color)
            .and_then(|set| Some(set.insert(spot)));
        locations.push(spot);
    }

    let mut ting: HashMap<(u8, V2i), HashSet<V2u>> = HashMap::new();
    for &offset in neighbourhood.iter() {
        for location in locations.iter() {
            if let Some(&color_a) = input.get(location.map(|e| e as isize)) {
                if let Some(color_b) = input.get(location.map(|e| e as isize) + offset) {
                    ting.entry((color_a, offset))
                        .or_default()
                        .union(&color_locations[color_b]);
                }
            }
        }
    }

    wave_map.resize(output_size, HashSet::from_iter(locations));

    let mut places_of_least_entropy = Vec::new();
    places_of_least_entropy.push(V2u {
        x: rng.gen_range(0..output_size.x),
        y: rng.gen_range(0..output_size.y),
    });

    while let Some(collapse_index) = places_of_least_entropy.choose(&mut rng) {
        // collapse superposition
        let &chosen = wave_map[collapse_index].iter().choose(&mut rng).unwrap();
        wave_map[collapse_index].clear();
        wave_map[collapse_index].insert(chosen);

        let chosen_color = input[chosen];

        // reduce entropy
        for &offset in neighbourhood.iter() {
            if let Some(constraining_set) = ting.get(&(chosen_color, offset)) {
                if let Some(reduced_set) =
                    wave_map.get_mut(collapse_index.map(|e| e as isize) + offset)
                {
                    *reduced_set = reduced_set
                        .intersection(constraining_set)
                        .copied()
                        .collect();
                }
            }
        }

        // find points of lowest entropy
        let mut lowest_entropy = usize::MAX;
        places_of_least_entropy.clear();
        for y in 0..wave_map.size.y {
            for x in 0..wave_map.size.x {
                let index = V2u::new(x, y);
                let entropy = wave_map[index].len();
                if entropy < lowest_entropy && entropy > 1 {
                    lowest_entropy = entropy;
                    places_of_least_entropy.clear();
                    places_of_least_entropy.push(index);
                } else if entropy == lowest_entropy {
                    places_of_least_entropy.push(index);
                }
            }
        }
        if lowest_entropy == 0 {
            return Err("WFC failed: there is a point with zero entropy".to_string());
        } else if lowest_entropy == 1 {
            break;
        }
    }
    let mut result = Grid::new();
    result.resize(output_size, 0);
    for (i, set) in wave_map.buf.iter().enumerate() {
        if set.len() > 1 {
            return Err("WFC failed: entropy is more than 1 somewhere".to_string());
        } else if set.len() < 1 {
            return Err("WFC failed: entropy is less than 1 somewhere".to_string());
        } else {
            result.buf[i] = input[set.iter().next().unwrap()];
        }
    }
    return Ok(result);
}
