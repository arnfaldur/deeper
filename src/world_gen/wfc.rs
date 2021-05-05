use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::num::NonZeroU32;
use std::ops::{Index, IndexMut};
use std::time::Instant;

use bit_set::BitSet;
use cgmath::{Array, Vector2, Zero};
use image::{DynamicImage, ImageBuffer, Pixel, Rgb, RgbImage};
use itertools::Itertools;
use rand::prelude::*;

pub fn test() {
    let timer = Instant::now();
    let pic = image::open("assets/Images/dungeon_sample.bmp").unwrap();
    let mut rng = StdRng::from_entropy();
    use wfc_image::*;
    if let Ok(out) = generate_image_with_rng(
        &pic,
        NonZeroU32::new(3).unwrap(),
        Size::new(256, 256),
        &[Orientation::Original], //more orientations = slower
        wrap::WrapNone,
        ForbidNothing,
        retry::NumTimes(40),
        &mut rng,
    ) {
        out.save("woah.png").unwrap();
        println!("Success!");
    }
    println!("elapsed {:?}", timer.elapsed());
}
