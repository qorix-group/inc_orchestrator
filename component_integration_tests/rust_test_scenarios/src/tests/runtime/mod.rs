mod channel;
mod execution_engine;
mod net;
mod sleep;
mod worker;

use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

use crate::tests::runtime::channel::channel_scenario_group;
use crate::tests::runtime::execution_engine::execution_engine_scenario_group;
use crate::tests::runtime::net::net_scenario_group;
use crate::tests::runtime::sleep::sleep_scenario_group;
use crate::tests::runtime::worker::worker_scenario_group;

pub fn runtime_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "runtime",
        vec![],
        vec![
            execution_engine_scenario_group(),
            sleep_scenario_group(),
            worker_scenario_group(),
            channel_scenario_group(),
            net_scenario_group(),
        ],
    ))
}
