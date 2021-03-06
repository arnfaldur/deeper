use std::time::{Instant, SystemTime};

use legion::{Resources, Schedule, World};

use crate::components::FrameTime;
use crate::debug::DebugTimer;
use crate::graphics::gui::GuiContext;
use crate::input::{Command, CommandManager, InputState};
use crate::systems;
use crate::systems::physics::PhysicsBuilderExtender;
use crate::transform::TransformBuilderExtender;

pub struct ECS {
    pub world: World,
    pub resources: Resources,

    schedules: Vec<(String, Schedule, Box<dyn Fn(&CommandManager) -> bool>)>,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            world: World::default(),
            resources: Resources::default(),
            schedules: vec![],
        }
    }

    pub fn create_schedules(&mut self) {
        self.schedules.push((
            "Engine Logic Schedule".into(),
            Schedule::builder()
                .add_system(systems::assets::hot_loading_system(
                    SystemTime::now(),
                    false,
                ))
                .add_system(systems::player::player_system())
                .add_system(systems::player::camera_control_system())
                .add_system(systems::world_gen::dung_gen_system())
                .add_system(systems::go_to_destination_system())
                .add_physics_systems(&mut self.world, &mut self.resources)
                .add_system(systems::spherical_offset_system())
                .add_transform_systems()
                .add_system(systems::assets::hot_loading_system(
                    SystemTime::now(),
                    false,
                ))
                .build(),
            Box::new(|command_manager| {
                command_manager.get(Command::DebugToggleLogic)
                    || command_manager.get(Command::DebugStepLogic)
            }),
        ));

        self.schedules.push((
            "Render Schedule".into(),
            systems::rendering::render_system_schedule(),
            Box::new(|_| true),
        ));
    }

    pub fn execute_schedules(&mut self) {
        let frame_time = self.resources.get::<Instant>().unwrap().elapsed();

        self.resources.insert(FrameTime(frame_time.as_secs_f32()));
        self.resources.insert(Instant::now());

        let mut debug_timer = DebugTimer::new();

        debug_timer.push("Frame");

        self.resources.insert(debug_timer);

        self.resources
            .get_mut::<GuiContext>()
            .unwrap()
            .prep_frame(&self.resources.get::<winit::window::Window>().unwrap());

        self.resources
            .get_mut::<CommandManager>()
            .unwrap()
            .update(&self.resources.get::<InputState>().unwrap());

        for (_, schedule, condition) in &mut self.schedules {
            let test: bool = {
                let command_manager = self
                    .resources
                    .get::<CommandManager>()
                    .expect("Schedule Execution requires a CommandManager in resources");

                (condition)(&command_manager)
            };
            if test {
                schedule.execute(&mut self.world, &mut self.resources);
            }
        }

        self.resources.get_mut::<InputState>().unwrap().new_frame();
    }
}