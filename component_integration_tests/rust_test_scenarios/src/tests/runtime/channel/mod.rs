mod spmc_broadcast;
mod spsc;

use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

fn spmc_broadcast_scenario_group() -> Box<dyn ScenarioGroup> {
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

fn spsc_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "spsc",
        vec![
            Box::new(spsc::SPSCSendReceive),
            Box::new(spsc::SPSCSendOnly),
            Box::new(spsc::SPSCDropReceiver),
            Box::new(spsc::SPSCDropSender),
            Box::new(spsc::SPSCDropSenderInTheMiddle),
            Box::new(spsc::SPSCDropReceiverInTheMiddle),
            Box::new(spsc::SPSCHeavyLoad),
        ],
        vec![],
    ))
}

pub fn channel_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "channel",
        vec![],
        vec![spsc_scenario_group(), spmc_broadcast_scenario_group()],
    ))
}
