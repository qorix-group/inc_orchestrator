pub mod sleep_basic;

use crate::internals::scenario::{ScenarioGroup, ScenarioGroupImpl};

pub struct SleepScenarioGroup {
    group: ScenarioGroupImpl,
}

impl SleepScenarioGroup {
    pub fn new() -> Self {
        SleepScenarioGroup {
            group: ScenarioGroupImpl::new("sleep"),
        }
    }
}

impl ScenarioGroup for SleepScenarioGroup {
    fn get_group_impl(&mut self) -> &mut ScenarioGroupImpl {
        &mut self.group
    }

    fn init(&mut self) -> () {
        self.group.add_scenario(Box::new(sleep_basic::SleepBasic));
    }
}
