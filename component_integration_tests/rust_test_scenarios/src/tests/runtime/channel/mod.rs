mod channel_spsc;

use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

pub fn channel_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "channel",
        vec![
            // SPSC channel tests
            Box::new(channel_spsc::SPSCSendReceive),
            Box::new(channel_spsc::SPSCSendOnly),
            Box::new(channel_spsc::SPSCDropReceiver),
            Box::new(channel_spsc::SPSCDropSender),
            Box::new(channel_spsc::SPSCDropSenderInTheMiddle),
            Box::new(channel_spsc::SPSCDropReceiverInTheMiddle),
            Box::new(channel_spsc::SPSCHeavyLoad),
        ],
        vec![],
    ))
}
