use std::time::Instant;

use assman::systems::AssetManagerBuilderExtender;
use entity_smith::FrameTime;
use enum_map::{Enum, EnumMap};
use graphics::debug::DebugTimer;
use graphics::gui::GuiRenderPipeline;
use graphics::systems::RenderBuilderExtender;
use input::{CommandManager, InputState};
use itertools::Itertools;
use legion::{Resources, Schedule, World};
use physics::PhysicsBuilderExtender;
use transforms::TransformBuilderExtender;

pub struct ScheduleEntry {
    schedule: Schedule,
}

impl Default for ScheduleEntry {
    fn default() -> Self {
        Self {
            schedule: Schedule::builder().build(),
        }
    }
}

/// Describes at what application stage a unit system bundle is run
#[derive(Enum)] // Needed for EnumMap
#[derive(Debug)]
pub enum UnitStage {
    /// Run once at initialization of the application
    Init,
    /// Run at the start of every frame
    StartFrame,
    /// Run every frame, precedes `UnitStage::Render`
    Logic,
    /// Run every frame, succeeds `UnitStage::Logic`
    Render,
    /// Run at the end of every frame
    EndFrame,
}

pub trait Unit {
    fn load_resources(&self, _world: &mut World, _resources: &mut Resources) { () }
    fn add_systems(&self, stage: UnitStage, builder: &mut SystemBuilder);
}

pub type SystemBuilder = legion::systems::Builder;

pub struct ApplicationBuilder {
    world: World,
    resources: Resources,
    schedule_builders: EnumMap<UnitStage, SystemBuilder>,
}

impl ApplicationBuilder {
    pub fn create_schedules(mut self) -> Self {
        self.schedule_builders[UnitStage::StartFrame].add_assman_systems();

        self.schedule_builders[UnitStage::Logic]
            .add_system(systems::player::player_system())
            .add_system(systems::player::camera_control_system())
            .add_system(world_gen::systems::dung_gen_system())
            .add_system(systems::go_to_destination_system())
            .add_physics_systems(&mut self.world, &mut self.resources)
            .add_transform_systems();

        self.schedule_builders[UnitStage::Render].add_render_systems();

        self
    }

    pub fn with_unit<T: Unit>(mut self, unit: T) -> Self {
        unit.load_resources(&mut self.world, &mut self.resources);
        self.schedule_builders
            .iter_mut()
            .map(|(stage, builder)| unit.add_systems(stage, builder))
            .count();

        self
    }

    pub fn build(mut self) -> Application {
        let mut schedules: EnumMap<UnitStage, ScheduleEntry> = EnumMap::default();

        schedules
            .iter_mut()
            .zip(
                self.schedule_builders
                    .values_mut()
                    .map(|f| ScheduleEntry {
                        schedule: f.build(),
                    })
                    .collect_vec()
                    .into_iter(),
            )
            .map(|((_, a), b)| *a = b)
            .count();

        Application {
            world: self.world,
            resources: self.resources,
            schedules,
        }
    }
}

pub struct Application {
    pub world: World,
    pub resources: Resources,
    schedules: EnumMap<UnitStage, ScheduleEntry>,
}

impl Application {
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder {
            world: World::default(),
            resources: Resources::default(),
            schedule_builders: EnumMap::default(),
        }
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

        for entry in self.schedules.values_mut() {
            entry.schedule.execute(&mut self.world, &mut self.resources);
        }

        self.resources.get_mut::<InputState>().unwrap().new_frame();
    }
}
