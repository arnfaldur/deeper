use std::cmp::max;
use std::fmt::{Debug, Error, Formatter};
use std::ops::{Index, IndexMut};

use cgmath::{Vector2, Zero};
use image::{ImageBuffer, Pixel};

pub type V2u = Vector2<usize>;
pub type V2i = Vector2<isize>;

#[derive(Clone)]
pub struct Grid<T> {
    pub(crate) size: V2u,
    pub(crate) buf: Vec<T>,
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
    pub(crate) fn to_1d_index(&self, index_2d: V2i) -> Option<isize> {
        return self
            .in_bounds(index_2d)
            .then(|| index_2d.y * self.size.x as isize + index_2d.x);
    }
    pub(crate) fn to_2d_index(&self, index_1d: isize) -> Option<V2i> {
        let result = V2i::new(
            index_1d % self.size.x as isize,
            index_1d / self.size.x as isize,
        );
        return self.in_bounds(result).then(|| result);
    }
}

impl<T: Debug> Debug for Grid<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut result = write!(f, "[\n");
        let mut padding = Vec::new();
        padding.resize(self.size.x, 0);
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let set = format!(
                    "{:?}, ",
                    self.get(V2i::new(x as isize, y as isize)).unwrap()
                );
                padding[x] = max(padding[x], set.len());
            }
        }
        padding.iter_mut().for_each(|pad| *pad += 4);
        for y in 0..self.size.y {
            result = result.and(write!(f, "\t",));
            for x in 0..self.size.x {
                let set = format!(
                    "{:?}, ",
                    self.get(V2i::new(x as isize, y as isize)).unwrap()
                );
                result = result.and(write!(f, "{:<1$}", set, padding[x]));
            }
            result = result.and(write!(f, "\n"));
        }

        result = result.and(write!(f, "]"));
        return result;
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
