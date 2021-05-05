#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;
    use std::time::Instant;

    use rand::prelude::StdRng;
    use rand::SeedableRng;
    use wfc_image::*;

    #[test]
    pub fn test() {
        let timer = Instant::now();
        let pic = image::open("assets/Images/dungeon_sample.bmp").unwrap();
        let mut rng = StdRng::from_entropy();
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
}
