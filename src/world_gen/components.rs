#[derive(Copy, Clone)]
pub enum MapTransition {
    None,
    Deeper, // Down to the next floor
}

pub struct MapSwitcher(pub MapTransition);

pub struct FloorNumber(pub i32);

#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
#[allow(unused)]
pub enum Faction {
    Enemies,
    Friends,
    Frenemies,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum TileType {
    Wall(Option<WallDirection>),
    Floor,
    Path,
    Nothing,
    LadderDown,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum WallDirection {
    North,
    West,
    South,
    East,
}
