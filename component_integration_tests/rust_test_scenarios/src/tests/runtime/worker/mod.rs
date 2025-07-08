pub mod worker_basic;
pub mod worker_with_blocking_tasks;

use crate::internals::scenario::{ScenarioGroup, ScenarioGroupImpl};
use worker_basic::BasicWorker;
use worker_with_blocking_tasks::WorkerWithBlockingTasks;

pub struct WorkerScenarioGroup {
    group: ScenarioGroupImpl,
}

impl WorkerScenarioGroup {
    pub fn new() -> Self {
        WorkerScenarioGroup {
            group: ScenarioGroupImpl::new("worker"),
        }
    }
}

impl ScenarioGroup for WorkerScenarioGroup {
    fn get_group_impl(&mut self) -> &mut ScenarioGroupImpl {
        &mut self.group
    }

    fn init(&mut self) -> () {
        self.group.add_scenario(Box::new(BasicWorker));
        self.group.add_scenario(Box::new(WorkerWithBlockingTasks));
    }
}
