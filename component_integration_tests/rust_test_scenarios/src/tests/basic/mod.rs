use orchestration::prelude::*;
use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};
use tracing::info;

mod only_shutdown_sequence;
mod program_runs;

fn simple_checkpoint(id: &str) {
    info!(id = id);
}

async fn basic_task() -> InvokeResult {
    simple_checkpoint("basic_task");
    Ok(())
}

pub fn basic_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "basic",
        vec![
            Box::new(only_shutdown_sequence::OnlyShutdownSequence),
            Box::new(program_runs::ProgramRun),
            Box::new(program_runs::ProgramRunMetered),
        ],
        vec![],
    ))
}
