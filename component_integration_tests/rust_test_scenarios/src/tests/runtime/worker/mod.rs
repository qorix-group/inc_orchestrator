mod num_workers;
mod worker_basic;
mod worker_with_blocking_tasks;

use num_workers::NumWorkers;
use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};
use worker_basic::BasicWorker;
use worker_with_blocking_tasks::WorkerWithBlockingTasks;

pub fn worker_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "worker",
        vec![Box::new(BasicWorker), Box::new(WorkerWithBlockingTasks), Box::new(NumWorkers)],
        vec![],
    ))
}
