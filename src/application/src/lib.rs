use std::time::Instant;

use entity_smith::FrameTime;
use graphics::debug::DebugTimer;
use graphics::gui::GuiRenderPipeline;
use input::{Command, CommandManager, InputState};
use legion::{Resources, Schedule, World};
use physics::PhysicsBuilderExtender;
use transforms::TransformBuilderExtender;

pub struct Application {
    pub world: World,
    pub resources: Resources,

    schedules: Vec<(String, Schedule, Box<dyn Fn(&CommandManager) -> bool>)>,
}

impl Application {
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
                .add_system(systems::player::player_system())
                .add_system(systems::player::camera_control_system())
                .add_system(world_gen::systems::dung_gen_system())
                .add_system(systems::go_to_destination_system())
                //.add_system(misc::SnakeSystem::new())
                .add_physics_systems(&mut self.world, &mut self.resources)
                .add_transform_systems()
                .build(),
            Box::new(|command_manager| {
                command_manager.get(Command::DebugToggleLogic)
                    || command_manager.get(Command::DebugStepLogic)
            }),
        ));

        self.schedules.push((
            "Asset Management Schedule".into(),
            assman::systems::assman_system_schedule(),
            Box::new(|_| true),
        ));

        self.schedules.push((
            "Render Schedule".into(),
            graphics::systems::render_system_schedule(),
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
            .get_mut::<GuiRenderPipeline>()
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
