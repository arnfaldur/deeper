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
