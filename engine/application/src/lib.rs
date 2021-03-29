use std::time::Instant;

use entity_smith::FrameTime;
use graphics::debug::DebugTimer;
use graphics::gui::GuiRenderPipeline;
use input::{Command, CommandManager, InputState};
use legion::{Resources, Schedule, World};
use physics::PhysicsBuilderExtender;
use transforms::TransformBuilderExtender;

type ScheduleCondition = Box<dyn Fn(&CommandManager) -> bool>;

pub struct ScheduleEntry {
    _label: String,
    schedule: Schedule,
    condition: Option<ScheduleCondition>,
}

impl ScheduleEntry {
    fn new(label: &str, schedule: Schedule, condition: Option<ScheduleCondition>) -> Self {
        Self {
            _label: label.to_string(),
            schedule,
            condition,
        }
    }
}

pub struct Application {
    pub world: World,
    pub resources: Resources,
    schedules: Vec<ScheduleEntry>,
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
        self.schedules.push(ScheduleEntry::new(
            "Engine Logic Schedule",
            Schedule::builder()
                .add_system(systems::player::player_system())
                .add_system(systems::player::camera_control_system())
                .add_system(world_gen::systems::dung_gen_system())
                .add_system(systems::go_to_destination_system())
                .add_system(misc::SnakeSystem::new())
                .add_physics_systems(&mut self.world, &mut self.resources)
                .add_transform_systems()
                .build(),
            Some(Box::new(|command_manager| {
                command_manager.get(Command::DebugToggleLogic)
                    || command_manager.get(Command::DebugStepLogic)
            })),
        ));

        self.schedules.push(ScheduleEntry::new(
            "Asset Management Schedule".into(),
            assman::systems::assman_system_schedule(),
            None,
        ));

        self.schedules.push(ScheduleEntry::new(
            "Render Schedule".into(),
            graphics::systems::render_system_schedule(),
            None,
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

        for entry in &mut self.schedules {
            let test = {
                let command_manager = self
                    .resources
                    .get::<CommandManager>()
                    .expect("Schedule Execution requires a CommandManager in resources");

                entry
                    .condition
                    .as_ref()
                    .map_or(true, |f| (f)(&command_manager))
            };
            if test {
                entry.schedule.execute(&mut self.world, &mut self.resources);
            }
        }

        self.resources.get_mut::<InputState>().unwrap().new_frame();
    }
}
