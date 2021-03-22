#![feature(map_first_last)]
#![feature(btree_retain)]

pub mod components;
mod dung_gen;
pub mod systems;
mod wfc;

// fixme: this should be removed
pub use wfc::test;
