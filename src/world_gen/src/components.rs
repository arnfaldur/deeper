use graphics::data::{LocalUniforms, Material};

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

// Graphics component requests.
// We want this to keep asset layer
// and world_gen layer very loosely
// coupled.

pub struct StaticModelRequest {
    pub label: String,
    pub uniforms: LocalUniforms,
}

pub struct DynamicModelRequest {
    pub label: String,
    pub material: Material,
    pub scale: f32,
}
