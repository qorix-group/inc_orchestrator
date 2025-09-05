mod spsc;

use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

pub fn spsc_scenario_group() -> Box<dyn ScenarioGroup> {
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
