use rand::Rng;
use wfc::{Coord, ForbidInterface, ForbidPattern, PatternId, Wrap};

#[derive(Clone)]
pub struct EmptyEdgesForbid {
    pub empty_tile_id: PatternId,
}

impl ForbidPattern for EmptyEdgesForbid {
    fn forbid<W: Wrap, R: Rng>(&mut self, fi: &mut ForbidInterface<W>, rng: &mut R) {
        for i in 0..(fi.wave_size().width() as i32) {
            let coord = Coord::new(i, fi.wave_size().height() as i32 - 1);
            fi.forbid_all_patterns_except(coord, self.empty_tile_id, rng)
                .unwrap();
            let coord = Coord::new(i, fi.wave_size().height() as i32 - 2);
            fi.forbid_all_patterns_except(coord, self.empty_tile_id, rng)
                .unwrap();
        }
        for i in 0..(fi.wave_size().height() as i32) {
            let coord = Coord::new(fi.wave_size().width() as i32 - 1, i);
            fi.forbid_all_patterns_except(coord, self.empty_tile_id, rng)
                .unwrap();
            let coord = Coord::new(fi.wave_size().width() as i32 - 2, i);
            fi.forbid_all_patterns_except(coord, self.empty_tile_id, rng)
                .unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;
    use std::time::Instant;

    use rand::prelude::StdRng;
    use rand::SeedableRng;
    use wfc_image::*;

    use crate::world_gen::wfc::EmptyEdgesForbid;

    #[test]
    pub fn test() {
        let timer = Instant::now();

        let map_size = Size::new(256, 256);

        let wfc_source = image::open("assets/Images/dungeon_sample.bmp").unwrap();

        let pattern_size = NonZeroU32::new(3).unwrap();
        let mut image_patterns =
            ImagePatterns::new(&wfc_source, pattern_size, &[Orientation::Original]);
        let top_left_corner_coord = Coord::new(0, 0);
        let top_left_corner_id = *image_patterns
            .id_grid_original_orientation()
            .get_checked(top_left_corner_coord);

        //image_patterns.pattern_mut(top_left_corner_id).clear_count();
        image_patterns.pattern_mut(top_left_corner_id);

        let mut rng = StdRng::from_entropy();
        if let Ok(out) = image_patterns
            .collapse_wave_retrying(
                map_size,
                wrap::WrapXY,
                EmptyEdgesForbid {
                    empty_tile_id: top_left_corner_id,
                },
                retry::NumTimes(40),
                &mut rng,
            )
            .map(|wave| image_patterns.image_from_wave(&wave))
        {
            out.save("woah.png").unwrap();
            println!("Success!");
        }
        println!("elapsed {:?}", timer.elapsed());
    }
}
