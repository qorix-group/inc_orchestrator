use super::orchestration_concurrency::{MultipleConcurrency, NestedConcurrency, SingleConcurrency};
use super::orchestration_sequence::{AwaitSequence, NestedSequence, SingleSequence};
use crate::internals::scenario::{ScenarioGroup, ScenarioGroupImpl};

pub struct OrchestrationScenarioGroup {
    group: ScenarioGroupImpl,
}

impl OrchestrationScenarioGroup {
    pub fn new() -> Self {
        OrchestrationScenarioGroup {
            group: ScenarioGroupImpl::new("orchestration"),
        }
    }
}

impl ScenarioGroup for OrchestrationScenarioGroup {
    fn get_group_impl(&mut self) -> &mut ScenarioGroupImpl {
        &mut self.group
    }

    fn init(&mut self) -> () {
        // Sequence scenarios
        self.group.add_scenario(Box::new(SingleSequence));
        self.group.add_scenario(Box::new(NestedSequence));
        self.group.add_scenario(Box::new(AwaitSequence));
        // Concurrency scenarios
        self.group.add_scenario(Box::new(SingleConcurrency));
        self.group.add_scenario(Box::new(MultipleConcurrency));
        self.group.add_scenario(Box::new(NestedConcurrency));
    }
}
