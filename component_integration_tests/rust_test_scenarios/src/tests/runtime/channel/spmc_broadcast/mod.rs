mod spmc_broadcast;

use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

pub fn spmc_broadcast_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "spmc_broadcast",
        vec![
            Box::new(spmc_broadcast::SPMCBroadcastSendReceive),
            Box::new(spmc_broadcast::SPMCBroadcastCreateReceiversOnly),
            Box::new(spmc_broadcast::SPMCBroadcastNumOfSubscribers),
            Box::new(spmc_broadcast::SPMCBroadcastDropAddReceiver),
            Box::new(spmc_broadcast::SPMCBroadcastSendReceiveOneLagging),
            Box::new(spmc_broadcast::SPMCBroadcastVariableReceivers),
            Box::new(spmc_broadcast::SPMCBroadcastDropSender),
            Box::new(spmc_broadcast::SPMCBroadcastHeavyLoad),
        ],
        vec![],
    ))
}
