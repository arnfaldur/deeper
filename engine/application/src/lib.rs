use enum_map::{Enum, EnumMap};
use itertools::Itertools;
use legion::{Resources, Schedule, World};

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
    fn load_resources(&self, _world: &mut World, _resources: &mut Resources) {}
    fn add_systems(&self, stage: UnitStage, builder: &mut SystemBuilder);
}

pub type SystemBuilder = legion::systems::Builder;

pub struct ApplicationBuilder {
    pub world: World,
    pub resources: Resources,
    pub schedule_builders: EnumMap<UnitStage, SystemBuilder>,
}

impl ApplicationBuilder {
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
        for entry in self.schedules.values_mut() {
            entry.schedule.execute(&mut self.world, &mut self.resources);
        }
    }
}
