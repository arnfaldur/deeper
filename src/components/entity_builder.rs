use legion::systems::CommandBuffer;
use legion::*;

pub struct EntityBuilder {
    command_buffer: CommandBuffer,
}
impl EntityBuilder {
    pub fn new(&mut self, world: &world::World) {
        self.command_buffer = CommandBuffer::new(world);
    }
    pub fn build(&mut self, world: &mut world::World) {
        self.command_buffer.flush(world);
    }
    pub fn dynamic_body(&self, speed: f32, acceleration: f32, mass: f32) -> &Self {
        self
    }
}
